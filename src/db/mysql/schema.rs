#![allow(proc_macro_derive_resolution_fallback)]

table! {
    batches (user_id, collection_id, id) {
        user_id -> Integer,
        collection_id -> Integer,
        id -> Varchar,
        modified -> Bigint,
        bsos -> Mediumtext,
    }
}

table! {
    bso (user_id, collection_id, id) {
        user_id -> Integer,
        collection_id -> Integer,
        id -> Varchar,
        sortindex -> Nullable<Integer>,
        payload -> Mediumtext,
        payload_size -> Integer,
        modified -> Bigint,
        expiry -> Bigint,
    }
}

table! {
    collections (id) {
        id -> Integer,
        name -> Varchar,
    }
}

table! {
    user_collections (user_id, collection_id) {
        user_id -> Integer,
        collection_id -> Integer,
        modified -> Bigint,
    }
}

allow_tables_to_appear_in_same_query!(batches, bso, collections, user_collections,);
