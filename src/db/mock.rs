//! Mock db implementation with methods stubbed to return default values.

use futures::future;

use super::*;

#[derive(Debug)]
pub struct MockDbPool;

impl MockDbPool {
    pub fn new() -> Self {
        MockDbPool
    }
}

impl DbPool for MockDbPool {
    fn get(&self) -> DbFuture<Box<dyn Db>> {
        Box::new(future::ok(Box::new(MockDb::new()) as Box<dyn Db>))
    }
}

#[derive(Clone, Debug)]
pub struct MockDb;

impl MockDb {
    pub fn new() -> Self {
        MockDb
    }
}

macro_rules! mock_db_method {
    ($name:ident, $type:ident) => {
        mock_db_method!($name, $type, results::$type);
    };
    ($name:ident, $type:ident, $result:ty) => {
        fn $name(&self, _params: params::$type) -> DbFuture<$result> {
            let result: $result = Default::default();
            Box::new(future::ok(result))
        }
    };
}

impl Db for MockDb {
    fn commit(&self) -> DbFuture<()> {
        Box::new(future::ok(()))
    }

    fn rollback(&self) -> DbFuture<()> {
        Box::new(future::ok(()))
    }

    fn box_clone(&self) -> Box<dyn Db> {
        Box::new(self.clone())
    }

    mock_db_method!(lock_for_read, LockCollection);
    mock_db_method!(lock_for_write, LockCollection);
    mock_db_method!(get_collection_modifieds, GetCollectionModifieds);
    mock_db_method!(get_collection_counts, GetCollectionCounts);
    mock_db_method!(get_collection_usage, GetCollectionUsage);
    mock_db_method!(get_storage_modified, GetStorageModified);
    mock_db_method!(get_storage_usage, GetStorageUsage);
    mock_db_method!(delete_storage, DeleteStorage);
    mock_db_method!(delete_collection, DeleteCollection);
    mock_db_method!(delete_bsos, DeleteBsos);
    mock_db_method!(get_bsos, GetBsos);
    mock_db_method!(post_bsos, PostBsos);
    mock_db_method!(delete_bso, DeleteBso);
    mock_db_method!(get_bso, GetBso, Option<results::GetBso>);
    mock_db_method!(put_bso, PutBso);
}

unsafe impl Send for MockDb {}
