use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

use diesel::r2d2;
use diesel::r2d2::Pool;

use futures::future::lazy;
use tokio_threadpool::ThreadPool;

use super::models::Result;
//#[cfg(test)]
//use super::test::TestTransactionCustomizer;
use crate::db::{error::DbError, Db, DbFuture, DbPool, STD_COLLS};
use crate::settings::Settings;

use super::manager::SpannerConnectionManager;
use super::models::SpannerDb;

embed_migrations!();

/// Run the diesel embedded migrations
///
/// Mysql DDL statements implicitly commit which could disrupt MysqlPool's
/// begin_test_transaction during tests. So this runs on its own separate conn.
//pub fn run_embedded_migrations(settings: &Settings) -> Result<()> {
//    let conn = MysqlConnection::establish(&settings.database_url)?;
//    Ok(embedded_migrations::run(&conn)?)
//}

#[derive(Clone)]
pub struct SpannerDbPool {
    /// Pool of db connections
    pool: Pool<SpannerConnectionManager>,
    /// Thread Pool for running synchronous db calls
    thread_pool: Arc<ThreadPool>,
    /// In-memory cache of collection_ids and their names
    coll_cache: Arc<CollectionCache>,
}

impl SpannerDbPool {
    /// Creates a new pool of Mysql db connections.
    ///
    /// Also initializes the Mysql db, ensuring all migrations are ran.
    pub fn new(settings: &Settings) -> Result<Self> {
        //run_embedded_migrations(settings)?;
        Self::new_without_migrations(settings)
    }

    pub fn new_without_migrations(settings: &Settings) -> Result<Self> {
        let m = SpannerConnectionManager::new(settings)?;
        let pool = r2d2::Pool::builder().build(m).unwrap();
        Ok(Self {
            pool,
            thread_pool: Arc::new(ThreadPool::new()),
            coll_cache: Default::default(),
        })
    }

    pub fn get_sync(&self) -> Result<SpannerDb> {
        Ok(SpannerDb::new(
            self.pool.get()?,
            Arc::clone(&self.thread_pool),
            Arc::clone(&self.coll_cache),
        ))
    }
}

impl DbPool for SpannerDbPool {
    fn get(&self) -> DbFuture<Box<dyn Db>> {
        let pool = self.clone();
        Box::new(self.thread_pool.spawn_handle(lazy(move || {
            pool.get_sync()
                .map(|db| Box::new(db) as Box<dyn Db>)
                .map_err(Into::into)
        })))
    }

    fn box_clone(&self) -> Box<dyn DbPool> {
        Box::new(self.clone())
    }
}

impl fmt::Debug for SpannerDbPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SpannerDbPool {{ coll_cache: {:?} }}", self.coll_cache)
    }
}

#[derive(Debug)]
pub struct CollectionCache {
    pub by_name: RwLock<HashMap<String, i32>>,
    pub by_id: RwLock<HashMap<i32, String>>,
}

impl CollectionCache {
    pub fn put(&self, id: i32, name: String) -> Result<()> {
        // XXX: should this emit a metric?
        // XXX: should probably either lock both simultaneously during
        // writes or use an RwLock alternative
        self.by_name
            .write()
            .map_err(|_| DbError::internal("by_name write"))?
            .insert(name.clone(), id);
        self.by_id
            .write()
            .map_err(|_| DbError::internal("by_id write"))?
            .insert(id, name);
        Ok(())
    }

    pub fn get_id(&self, name: &str) -> Result<Option<i32>> {
        Ok(self
            .by_name
            .read()
            .map_err(|_| DbError::internal("by_name read"))?
            .get(name)
            .cloned())
    }

    pub fn get_name(&self, id: i32) -> Result<Option<String>> {
        Ok(self
            .by_id
            .read()
            .map_err(|_| DbError::internal("by_id read"))?
            .get(&id)
            .cloned())
    }
}

impl Default for CollectionCache {
    fn default() -> Self {
        Self {
            by_name: RwLock::new(
                STD_COLLS
                    .iter()
                    .map(|(k, v)| ((*v).to_owned(), *k))
                    .collect(),
            ),
            by_id: RwLock::new(
                STD_COLLS
                    .iter()
                    .map(|(k, v)| (*k, (*v).to_owned()))
                    .collect(),
            ),
        }
    }
}
