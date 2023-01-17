
use crate::LoadingConfig;
use crate::package::{CorePackage, Package};
use melodium_common::descriptor::{Collection, Context, Entry, Function, Identifier, Loader as LoaderTrait, LoadingError, Model, Treatment, collection};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub struct Loader {
    collection: RwLock<Collection>,
    packages: RwLock<HashMap<String, Box<dyn Package>>>,
    search_locations: Vec<PathBuf>,
}

impl Loader {

    pub fn new(config: LoadingConfig) -> Self {
        Self {
            collection: RwLock::new(Collection::new()),
            packages: RwLock::new(config.core_packages.into_iter().map(|p| (p.name().to_string(), Box::new(CorePackage::new(p)) as Box<dyn Package>)).collect()),
            search_locations: config.search_locations,
        }
    }

    pub fn load(&mut self, identifier: &Identifier) -> Result<Collection, LoadingError> {

    }

    fn get_with_load(&self, identifier: &Identifier) -> Result<Entry, LoadingError> {
        
        if let Some(entry) = self.collection.read().unwrap().get(identifier) {
            Ok(entry.clone())
        } else if let Some(package) = self.packages.read().unwrap().get(identifier.root()) {

            let additions = package.element(self, identifier)?;
            self.add_collection(additions);

            Ok(self.collection.read().unwrap().get(identifier).unwrap().clone())
        } else {

        }
    }

    fn add_collection(&self, collection: Collection) {

    }
}

impl LoaderTrait for Loader {
    fn load_context(&self, identifier: &Identifier) -> Result<Arc<Context>, LoadingError> {
        
    }

    fn load_function(&self, identifier: &Identifier) -> Result<Arc<dyn Function>, LoadingError> {
        todo!()
    }

    fn load_model(&self, identifier: &Identifier) -> Result<Arc<dyn Model>, LoadingError> {
        todo!()
    }

    fn load_treatment(&self, identifier: &Identifier) -> Result<Arc<dyn Treatment>, LoadingError> {
        todo!()
    }
}
