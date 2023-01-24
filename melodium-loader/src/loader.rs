
use crate::LoadingConfig;
use crate::package::{CorePackage, Package};
use melodium_common::descriptor::{Collection, Context, Entry, Function, Identifier, Loader as LoaderTrait, LoadingError, Model, Treatment};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard};

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

    pub fn load(&self, identifier: &Identifier) -> Result<Collection, LoadingError> {
        self.get_with_load(identifier)?;
        Ok(self.collection.read().unwrap().clone())
    }

    pub fn collection(&self) -> RwLockReadGuard<Collection> {
        self.collection.read().unwrap()
    }

    pub fn get_with_load(&self, identifier: &Identifier) -> Result<Entry, LoadingError> {
        
        if let Some(entry) = self.collection.read().unwrap().get(identifier) {
            Ok(entry.clone())
        } else if let Some(package) = self.packages.read().unwrap().get(identifier.root()) {

            let additions = package.element(self, identifier)?;
            self.add_collection(additions);

            Ok(self.collection.read().unwrap().get(identifier).unwrap().clone())
        } else {
            Err(LoadingError::NoPackage)
        }
    }

    fn add_collection(&self, other_collection: Collection) {

        let existing = self.collection.read().unwrap().identifiers();
        let mut others = other_collection.identifiers();

        others.retain(|id| !existing.contains(id));

        if !others.is_empty() {
            let mut collection = self.collection.write().unwrap();
            for id in &others {
                collection.insert(other_collection.get(id).unwrap().clone());
            }
        }
    }
}

impl LoaderTrait for Loader {
    fn load_context(&self, identifier: &Identifier) -> Result<Arc<Context>, LoadingError> {
        match self.get_with_load(identifier)? {
            Entry::Context(context) => Ok(context),
            _ => Err(LoadingError::ContextExpected)
        }
    }

    fn load_function(&self, identifier: &Identifier) -> Result<Arc<dyn Function>, LoadingError> {
        match self.get_with_load(identifier)? {
            Entry::Function(function) => Ok(function),
            _ => Err(LoadingError::FunctionExpected)
        }
    }

    fn load_model(&self, identifier: &Identifier) -> Result<Arc<dyn Model>, LoadingError> {
        match self.get_with_load(identifier)? {
            Entry::Model(model) => Ok(model),
            _ => Err(LoadingError::ModelExpected)
        }
    }

    fn load_treatment(&self, identifier: &Identifier) -> Result<Arc<dyn Treatment>, LoadingError> {
        match self.get_with_load(identifier)? {
            Entry::Treatment(treatment) => Ok(treatment),
            _ => Err(LoadingError::TreatmentExpected)
        }
    }
}
