
use std::sync::Arc;
use std::path::PathBuf;
use crate::logic::collection_pool::CollectionPool;
pub struct Documentation {
    pub roots: Vec<String>,
    pub collection: Arc<CollectionPool>,
    pub output_path: PathBuf,
}

impl Documentation {

    pub fn new(roots: Vec<String>, collection: Arc<CollectionPool>, output_path: PathBuf) -> Self {
        Self {
            roots,
            collection,
            output_path,
        }
    }
}
