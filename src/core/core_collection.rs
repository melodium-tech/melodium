
use crate::logic::collection_pool::CollectionPool;

pub fn core_collection() -> &'static CollectionPool {

    lazy_static! {
        static ref SINGLETON: CollectionPool = CollectionPool::new();
    }
    &SINGLETON
}