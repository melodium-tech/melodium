use melodium_common::descriptor::Package;
use std::path::PathBuf;

pub struct LoadingConfig {
    pub core_packages: Vec<Box<dyn Package>>,
    pub search_locations: Vec<PathBuf>,
}
