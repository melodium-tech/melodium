use crate::package::PackageTrait as Package;
use crate::package_manager::{PackageManager, PackageManagerConfiguration};
use crate::{LoadingConfig, PackageInfo};
use melodium_common::descriptor::{
    Collection, Context, Entry, Function, Identified, Identifier, IdentifierRequirement,
    Loader as LoaderTrait, LoadingError, LoadingResult, Model, PackageRequirement, Treatment,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};

/**
 * Manages loading of Mélodium packages.
 *
 * The loader take care of managing package dependencies, loading inner elements of packages,
 * and building a coherent [Collection].
 *
 * Loading can be made through the `load`-ing functions, and then final collection rendered using `build()`.
 *
 * This loader aims to be lazy, please read carefully the behavior of each `load`-ing function in order to make best
 * use of them.
 */
#[derive(Debug)]
pub struct Loader {
    collection: RwLock<Collection>,
    package_manager: PackageManager,
}

impl Loader {
    pub fn new(config: LoadingConfig) -> Self {
        Self {
            collection: RwLock::new(Collection::new()),
            package_manager: PackageManager::new(PackageManagerConfiguration {
                repositories: vec![
                    #[cfg(feature = "filesystem")]
                    Arc::new(std::sync::Mutex::new(melodium_repository::Repository::new(
                        melodium_repository::RepositoryConfig {
                            repository_location: {
                                let mut path = std::env::var_os("MELODIUM_HOME")
                                    .map(|var| var.into())
                                    .or_else(|| {
                                        simple_home_dir::home_dir().map(|mut path| {
                                            path.push(".melodium");
                                            path
                                        })
                                    })
                                    .unwrap_or_else(|| {
                                        let mut path = std::env::temp_dir();
                                        path.push("melodium");
                                        path
                                    });
                                path.push(env!("CARGO_PKG_VERSION"));
                                path
                            },
                            network: if cfg!(feature = "network") {
                                Some(melodium_repository::network::NetworkRepositoryConfiguration::new())
                            } else {
                                None
                            },
                        },
                    ))),
                ],
                core_packages: config.core_packages,
                search_locations: config.search_locations,
                raw_elements: config.raw_elements,
                allow_network: cfg!(feature = "network"),
            }),
        }
    }

    /**
     * Loads the given package, according to requirements.
     *
     * This function _does not_ load any package content on its own, see [Self::load], [Self::load_all] or the functions of [LoaderTrait]
     * to get elements required loaded.
     */
    pub fn load_package(
        &self,
        requirement: &PackageRequirement,
    ) -> LoadingResult<Arc<dyn PackageInfo>> {
        self.package_manager
            .get_package(requirement)
            .and_then(|pkg| LoadingResult::new_success(Arc::clone(&pkg) as Arc<dyn PackageInfo>))
    }

    /**
     * Loads the given raw package content.
     *
     * This function _does not_ load any package content on its own, see [Self::load], [Self::load_all] or the functions of [LoaderTrait]
     * to get elements required loaded.
     */
    pub fn load_raw(&self, raw_content: Arc<Vec<u8>>) -> LoadingResult<Arc<dyn PackageInfo>> {
        self.package_manager
            .add_raw_package(raw_content)
            .and_then(|pkg| LoadingResult::new_success(Arc::clone(&pkg) as Arc<dyn PackageInfo>))
    }

    /**
     * Loads the given mapped package content.
     *
     * This function _does not_ load any package content on its own, see [Self::load], [Self::load_all] or the functions of [LoaderTrait]
     * to get elements required loaded.
     */
    pub fn load_mapped(
        &self,
        mapped_content: HashMap<String, Vec<u8>>,
    ) -> LoadingResult<Arc<dyn PackageInfo>> {
        self.package_manager
            .add_map_package(mapped_content)
            .and_then(|pkg| LoadingResult::new_success(Arc::clone(&pkg) as Arc<dyn PackageInfo>))
    }

    /**
     * Load the given identifier.
     */
    pub fn load(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Identifier> {
        self.get_with_load(identifier_requirement)
            .and_then(|entry| LoadingResult::new_success(entry.identifier().clone()))
    }

    /**
     * Load all the elements from all the packages.
     *
     * Packages concerned have to be already loaded through [Self::load_package] of [Self::load_raw] functions.
     */
    pub fn load_all(&self) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for package in self.package_manager.get_packages() {
            if let Some(additions) = result.merge_degrade_failure(package.full_collection(self)) {
                self.add_collection(additions);
            }
        }
        result
    }

    /**
     * Proceed to build of coherent collection.
     */
    pub fn build(&self) -> LoadingResult<Arc<Collection>> {
        let mut result = LoadingResult::new_success(());
        let collection = Arc::new(self.collection.read().unwrap().clone());

        for package in self.package_manager.get_packages() {
            result.merge_degrade_failure(package.make_building(&collection));
        }

        result.and(LoadingResult::new_success(collection))
    }

    pub fn packages(&self) -> Vec<Arc<dyn PackageInfo>> {
        self.package_manager
            .get_packages()
            .into_iter()
            .map(|pkg| Arc::clone(&pkg) as Arc<dyn PackageInfo>)
            .collect()
    }

    pub fn collection(&self) -> RwLockReadGuard<Collection> {
        self.collection.read().unwrap()
    }

    pub fn get_with_load(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Entry> {
        let mut result = LoadingResult::new_success(());
        let entry = self
            .collection
            .read()
            .unwrap()
            .get(identifier_requirement)
            .cloned();
        if let Some(entry) = entry {
            result.and_degrade_failure(LoadingResult::new_success(entry))
        } else if let Some(package) = result.merge_degrade_failure(
            self.package_manager
                .get_package(&identifier_requirement.package_requirement()),
        ) {
            package
                .element(self, &identifier_requirement)
                .and_then(|additions| {
                    self.add_collection(additions);
                    if let Some(element) = self
                        .collection
                        .read()
                        .unwrap()
                        .get(identifier_requirement)
                        .cloned()
                    {
                        result.and_degrade_failure(LoadingResult::new_success(element))
                    } else {
                        result.and_degrade_failure(LoadingResult::new_failure(
                            LoadingError::not_found(249, identifier_requirement.to_string()),
                        ))
                    }
                })
        } else {
            result.and_degrade_failure(LoadingResult::new_failure(LoadingError::no_package(
                167,
                identifier_requirement.package_requirement(),
            )))
        }
    }

    fn add_collection(&self, other_collection: Collection) {
        let existing = self.collection.read().unwrap().identifiers();
        let mut others = other_collection.identifiers();

        others.retain(|id| !existing.contains(id));

        if !others.is_empty() {
            let mut collection = self.collection.write().unwrap();
            for id in &others {
                collection.insert(other_collection.get(&id.into()).unwrap().clone());
            }
        }
    }
}

impl LoaderTrait for Loader {
    fn load_context(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Arc<dyn Context>> {
        self.get_with_load(identifier_requirement)
            .and_then(|entry| match entry {
                Entry::Context(context) => LoadingResult::new_success(context),
                _ => LoadingResult::new_failure(LoadingError::context_expected(
                    168,
                    None,
                    identifier_requirement.clone(),
                )),
            })
    }

    fn load_function(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Arc<dyn Function>> {
        self.get_with_load(identifier_requirement)
            .and_then(|entry| match entry {
                Entry::Function(function) => LoadingResult::new_success(function),
                _ => LoadingResult::new_failure(LoadingError::function_expected(
                    169,
                    None,
                    identifier_requirement.clone(),
                )),
            })
    }

    fn load_model(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Arc<dyn Model>> {
        self.get_with_load(identifier_requirement)
            .and_then(|entry| match entry {
                Entry::Model(model) => LoadingResult::new_success(model),
                _ => LoadingResult::new_failure(LoadingError::model_expected(
                    170,
                    None,
                    identifier_requirement.clone(),
                )),
            })
    }

    fn load_treatment(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Arc<dyn Treatment>> {
        self.get_with_load(identifier_requirement)
            .and_then(|entry| match entry {
                Entry::Treatment(treatment) => LoadingResult::new_success(treatment),
                _ => LoadingResult::new_failure(LoadingError::treatment_expected(
                    171,
                    None,
                    identifier_requirement.clone(),
                )),
            })
    }
}
