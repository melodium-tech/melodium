
use melodium_common::descriptor::Package;
use std::path::PathBuf;
use std::sync::Arc;

pub struct LoadingContext {
    pub core_packages: Vec<Arc<dyn Package>>,
    pub search_locations: Vec<PathBuf>,
}
