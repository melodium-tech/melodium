use crate::package::PackageTrait as Package;
use crate::package_manager::{PackageManager, PackageManagerConfiguration};
use crate::LoadingConfig;
use melodium_common::descriptor::{
    Collection, Context, Entry, Function, Identifier, Loader as LoaderTrait, LoadingError,
    LoadingResult, Model, PackageRequirement, Treatment, VersionReq,
};
use melodium_repository::network::NetworkRepositoryConfiguration;
use melodium_repository::{Repository, RepositoryConfig};
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

/**
 * Manages loading of Mélodium packages.
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
                repositories: vec![Arc::new(Mutex::new(Repository::new(RepositoryConfig {
                    repository_location: {
                        let mut path = std::env::var_os("MELODIUM_HOME")
                            .or_else(|| {
                                std::env::var_os("HOME").map(|home| {
                                    let mut path = home.to_os_string();
                                    path.push("/.melodium");
                                    path
                                })
                            })
                            .unwrap_or("/tmp/melodium".into());
                        path.push("/");
                        path.push(env!("CARGO_PKG_VERSION"));
                        path
                    }
                    .into(),
                    network: if cfg!(feature = "network") {
                        Some(NetworkRepositoryConfiguration::new())
                    } else {
                        None
                    },
                })))],
                core_packages: config.core_packages,
                search_locations: config.search_locations,
                raw_elements: config.raw_elements,
                allow_network: cfg!(feature = "network"),
            }),
        }
    }

    pub fn load_package(&self, requirement: &PackageRequirement) -> LoadingResult<()> {
        self.package_manager
            .get_package(requirement)
            .and(LoadingResult::new_success(()))
    }

    pub fn load_raw(
        &self,
        raw_content: Arc<Vec<u8>>,
    ) -> LoadingResult<(String, Option<Identifier>)> {
        self.package_manager
            .add_raw_package(raw_content)
            .and_then(|pkg| {
                LoadingResult::new_success((pkg.name().to_string(), pkg.main().clone()))
            })
    }

    pub fn load(&self, identifier: &Identifier) -> LoadingResult<()> {
        self.get_with_load(identifier)
            .and_then(|_| LoadingResult::new_success(()))
    }

    pub fn load_all(&self) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for package in self.package_manager.get_packages() {
            if let Some(additions) = result.merge_degrade_failure(package.full_collection(self)) {
                self.add_collection(additions);
            }
        }
        result
    }

    pub fn build(&self) -> LoadingResult<Arc<Collection>> {
        let mut result = LoadingResult::new_success(());
        let collection = Arc::new(self.collection.read().unwrap().clone());

        for package in self.package_manager.get_packages() {
            result.merge_degrade_failure(package.make_building(&collection));
        }

        result.and(LoadingResult::new_success(collection))
    }

    pub fn collection(&self) -> RwLockReadGuard<Collection> {
        self.collection.read().unwrap()
    }

    pub fn get_with_load(&self, identifier: &Identifier) -> LoadingResult<Entry> {
        let mut result = LoadingResult::new_success(());
        let entry = self.collection.read().unwrap().get(identifier).cloned();
        if let Some(entry) = entry {
            result.and_degrade_failure(LoadingResult::new_success(entry))
        } else if let Some(package) =
            result.merge_degrade_failure(self.package_manager.get_package(&PackageRequirement {
                package: identifier.root().to_string(),
                // This is a development-phase tradeoff, see https://gitlab.com/melodium/melodium/-/issues/62
                version_requirement:
                    VersionReq::parse(&format!("={}", env!("CARGO_PKG_VERSION"))).unwrap(),
            }))
        {
            package.element(self, identifier).and_then(|additions| {
                self.add_collection(additions);
                result.and_degrade_failure(LoadingResult::new_success(
                    self.collection
                        .read()
                        .unwrap()
                        .get(identifier)
                        .unwrap()
                        .clone(),
                ))
            })
        } else {
            result.and_degrade_failure(LoadingResult::new_failure(LoadingError::no_package(
                167,
                identifier.root().to_string(),
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
