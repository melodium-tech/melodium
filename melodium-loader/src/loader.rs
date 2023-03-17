#[cfg(feature = "filesystem")]
use crate::package::FsPackage;
use crate::package::{CorePackage, Package};
use crate::LoadingConfig;
use melodium_common::descriptor::{
    Collection, Context, Entry, Function, Identifier, Loader as LoaderTrait, LoadingError, Model,
    Treatment,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard};

/**
 * Manages loading of Mélodium packages.
 */
#[derive(Debug)]
pub struct Loader {
    collection: RwLock<Collection>,
    packages: RwLock<HashMap<String, Box<dyn Package>>>,
    search_locations: Vec<PathBuf>,
}

impl Loader {
    pub fn new(config: LoadingConfig) -> Self {
        Self {
            collection: RwLock::new(Collection::new()),
            packages: RwLock::new(
                config
                    .core_packages
                    .into_iter()
                    .map(|p| {
                        (
                            p.name().to_string(),
                            Box::new(CorePackage::new(p)) as Box<dyn Package>,
                        )
                    })
                    .collect(),
            ),
            search_locations: config.search_locations,
        }
    }

    pub fn load_package(&self, name: &str) -> Result<(), LoadingError> {
        if !self.packages.read().unwrap().contains_key(name) {
            for location in &self.search_locations {
                let mut path = location.clone();
                path.push(name);
                if path.exists() {
                    #[cfg(feature = "filesystem")]
                    if let Ok(package) = FsPackage::new(&path) {
                        for req in package.requirements() {
                            self.load_package(req)?;
                        }
                        self.packages
                            .write()
                            .unwrap()
                            .insert(name.to_string(), Box::new(package));
                        return Ok(());
                    }
                }
            }
            Err(LoadingError::NoPackage)
        } else {
            Ok(())
        }
    }

    pub fn load(&self, identifier: &Identifier) -> Result<Collection, LoadingError> {
        self.get_with_load(identifier)?;
        Ok(self.collection.read().unwrap().clone())
    }

    pub fn full_load(&self) -> Result<Collection, LoadingError> {
        for (_, package) in self.packages.read().unwrap().iter() {
            let additions = package.full_collection(self)?;
            self.add_collection(additions);
        }
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

            Ok(self
                .collection
                .read()
                .unwrap()
                .get(identifier)
                .unwrap()
                .clone())
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
    fn load_context(&self, identifier: &Identifier) -> Result<Arc<dyn Context>, LoadingError> {
        match self.get_with_load(identifier)? {
            Entry::Context(context) => Ok(context),
            _ => Err(LoadingError::ContextExpected),
        }
    }

    fn load_function(&self, identifier: &Identifier) -> Result<Arc<dyn Function>, LoadingError> {
        match self.get_with_load(identifier)? {
            Entry::Function(function) => Ok(function),
            _ => Err(LoadingError::FunctionExpected),
        }
    }

    fn load_model(&self, identifier: &Identifier) -> Result<Arc<dyn Model>, LoadingError> {
        match self.get_with_load(identifier)? {
            Entry::Model(model) => Ok(model),
            _ => Err(LoadingError::ModelExpected),
        }
    }

    fn load_treatment(&self, identifier: &Identifier) -> Result<Arc<dyn Treatment>, LoadingError> {
        match self.get_with_load(identifier)? {
            Entry::Treatment(treatment) => Ok(treatment),
            _ => Err(LoadingError::TreatmentExpected),
        }
    }
}
