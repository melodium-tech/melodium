use melodium_common::descriptor::Package;
use std::path::PathBuf;
use std::sync::Arc;

/**
 * Provides base loading configuration.
 *
 * The configuration behavior depends on features that are activated on build.
 * The `search_locations` field is inspected in order.
 */
#[derive(Debug, Default)]
pub struct LoadingConfig {
    /// List of built-in packages loading procedure can rely on
    pub core_packages: Vec<Arc<dyn Package>>,
    /// Locations where loading procedure can look to get packages
    pub search_locations: Vec<PathBuf>,
    pub raw_elements: Vec<Arc<Vec<u8>>>,
}

impl LoadingConfig {
    pub fn new() -> Self {
        Self {
            core_packages: Vec::new(),
            search_locations: Vec::new(),
            raw_elements: Vec::new(),
        }
    }

    pub fn extend(&mut self, mut other: Self) {
        self.core_packages.append(&mut other.core_packages);
        self.search_locations.append(&mut other.search_locations);
        self.raw_elements.append(&mut other.raw_elements);
    }
}
