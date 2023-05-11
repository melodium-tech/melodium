#[cfg(feature = "filesystem")]
use crate::package::FsPackage;
use crate::package::{CorePackage, Package, RawPackage};
use crate::LoadingConfig;
use melodium_common::descriptor::{
    Collection, Context, Entry, Function, Identifier, Loader as LoaderTrait, LoadingError,
    LoadingResult, Model, Treatment,
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

    pub fn load_package(&self, name: &str) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        if !self.packages.read().unwrap().contains_key(name) {
            let mut found = false;
            for location in &self.search_locations {
                let mut path = location.clone();
                path.push(name);
                if path.exists() {
                    #[cfg(feature = "filesystem")]
                    if let Some(package) = result.merge_degrade_failure(FsPackage::new(&path)) {
                        for req in package.requirements() {
                            result.merge_degrade_failure(self.load_package(req));
                        }
                        self.packages
                            .write()
                            .unwrap()
                            .insert(name.to_string(), Box::new(package));
                        found = true;
                    }
                }
            }
            if !found {
                result = result.and_degrade_failure(LoadingResult::new_failure(
                    LoadingError::no_package(166, name.to_string()),
                ));
            }
        }
        result
    }

    pub fn load_raw(&self, raw_content: &str) -> LoadingResult<String> {
        RawPackage::new(raw_content).and_then(|package| {
            let name = package.name().to_string();

            self.packages
                .write()
                .unwrap()
                .insert(package.name().to_string(), Box::new(package));

            LoadingResult::new_success(name)
        })
    }

    pub fn load(&self, identifier: &Identifier) -> LoadingResult<()> {
        self.get_with_load(identifier)
            .and_then(|_| LoadingResult::new_success(()))
    }

    pub fn load_all(&self) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for (_name, package) in self.packages.read().unwrap().iter() {
            if let Some(additions) = result.merge_degrade_failure(package.full_collection(self)) {
                self.add_collection(additions);
            }
        }
        result
    }

    pub fn build(&self) -> LoadingResult<Arc<Collection>> {
        let mut result = LoadingResult::new_success(());
        let collection = Arc::new(self.collection.read().unwrap().clone());

        for (_name, package) in self.packages.read().unwrap().iter() {
            result.merge_degrade_failure(package.make_building(&collection));
        }

        result.and(LoadingResult::new_success(collection))
    }

    pub fn collection(&self) -> RwLockReadGuard<Collection> {
        self.collection.read().unwrap()
    }

    pub fn get_with_load(&self, identifier: &Identifier) -> LoadingResult<Entry> {
        let entry = self.collection.read().unwrap().get(identifier).cloned();
        if let Some(entry) = entry {
            LoadingResult::new_success(entry)
        } else if let Some(package) = self.packages.read().unwrap().get(identifier.root()) {
            package.element(self, identifier).and_then(|additions| {
                self.add_collection(additions);
                LoadingResult::new_success(
                    self.collection
                        .read()
                        .unwrap()
                        .get(identifier)
                        .unwrap()
                        .clone(),
                )
            })
        } else {
            LoadingResult::new_failure(LoadingError::no_package(167, identifier.root().to_string()))
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
    fn load_context(&self, identifier: &Identifier) -> LoadingResult<Arc<dyn Context>> {
        self.get_with_load(identifier)
            .and_then(|entry| match entry {
                Entry::Context(context) => LoadingResult::new_success(context),
                _ => LoadingResult::new_failure(LoadingError::context_expected(
                    168,
                    None,
                    identifier.clone(),
                )),
            })
    }

    fn load_function(&self, identifier: &Identifier) -> LoadingResult<Arc<dyn Function>> {
        self.get_with_load(identifier)
            .and_then(|entry| match entry {
                Entry::Function(function) => LoadingResult::new_success(function),
                _ => LoadingResult::new_failure(LoadingError::function_expected(
                    169,
                    None,
                    identifier.clone(),
                )),
            })
    }

    fn load_model(&self, identifier: &Identifier) -> LoadingResult<Arc<dyn Model>> {
        self.get_with_load(identifier)
            .and_then(|entry| match entry {
                Entry::Model(model) => LoadingResult::new_success(model),
                _ => LoadingResult::new_failure(LoadingError::model_expected(
                    170,
                    None,
                    identifier.clone(),
                )),
            })
    }

    fn load_treatment(&self, identifier: &Identifier) -> LoadingResult<Arc<dyn Treatment>> {
        self.get_with_load(identifier)
            .and_then(|entry| match entry {
                Entry::Treatment(treatment) => LoadingResult::new_success(treatment),
                _ => LoadingResult::new_failure(LoadingError::treatment_expected(
                    171,
                    None,
                    identifier.clone(),
                )),
            })
    }
}
