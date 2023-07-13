use crate::compo::Compo;
use crate::content::Content;
use crate::package::package::PackageTrait;
use crate::Loader;
use glob::{glob_with, MatchOptions};
use melodium_common::descriptor::{
    Collection, Identifier, LoadingError, LoadingResult, PackageRequirement, Version,
};
use std::collections::HashMap;
use std::fs::{metadata, read, read_to_string};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard};

#[derive(Debug)]
pub struct FsPackage {
    path: PathBuf,
    name: String,
    version: Version,
    requirements: Vec<PackageRequirement>,
    main: Option<Identifier>,
    contents: RwLock<HashMap<PathBuf, Content>>,
}

impl FsPackage {
    pub fn new(path: &Path) -> LoadingResult<Self> {
        let location = match metadata(path) {
            Ok(location) => location,
            Err(_) => {
                return LoadingResult::new_failure(LoadingError::no_package(
                    176,
                    path.to_string_lossy().to_string(),
                ))
            }
        };

        if location.is_dir() {
            let mut composition_path = path.to_path_buf();
            composition_path.push("Compo.toml");

            let composition = match read_to_string(&composition_path) {
                Ok(location) => location,
                Err(_) => {
                    return LoadingResult::new_failure(LoadingError::no_package(
                        177,
                        path.to_string_lossy().to_string(),
                    ))
                }
            };

            let mut result = LoadingResult::new_success(());
            if let Some(composition) = result.merge_degrade_failure(Compo::parse(&composition)) {
                result.and(LoadingResult::new_success(Self {
                    path: path.to_path_buf(),
                    name: composition.name.clone(),
                    main: composition.main.clone().map(|id| {
                        let mut path = vec![composition.name.clone()];
                        path.extend(id.path().iter().map(|s| s.clone()));
                        Identifier::new(path, id.name())
                    }),
                    version: composition.version,
                    requirements: composition.requirements,
                    contents: RwLock::new(HashMap::new()),
                }))
            } else {
                return result.and_degrade_failure(LoadingResult::new_failure(
                    LoadingError::no_package(182, path.to_string_lossy().to_string()),
                ));
            }
        } else {
            return LoadingResult::new_failure(LoadingError::no_package(
                183,
                path.to_string_lossy().to_string(),
            ));
        }
    }

    fn insure_content(&self, designation: &Path) -> LoadingResult<()> {
        if self.contents.read().unwrap().contains_key(designation) {
            return LoadingResult::new_success(());
        }

        let mut full_path = self.path.clone();
        full_path.push(designation);
        let raw = match read(full_path) {
            Ok(val) => val,
            Err(_) => {
                return LoadingResult::new_failure(LoadingError::no_package(
                    182,
                    designation.to_string_lossy().to_string(),
                ))
            }
        };

        let result_content = Content::new(
            &format!(
                "{}/{}",
                self.name,
                designation.as_os_str().to_string_lossy()
            ),
            &raw,
        );

        result_content
            .convert_failure_errors(|err| LoadingError::content_error(183, Arc::new(err)))
            .and_then(|content| {
                self.contents
                    .write()
                    .unwrap()
                    .insert(designation.to_path_buf(), content);
                LoadingResult::new_success(())
            })
    }

    pub fn all_contents(&self) -> LoadingResult<()> {
        let pattern = format!("{}/**/*.mel", self.path.to_string_lossy().to_string());

        let options = MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: true,
        };

        let mut result = LoadingResult::new_success(());
        if let Some(paths) = result.merge_degrade_failure(match glob_with(&pattern, options) {
            Ok(paths) => LoadingResult::new_success(paths),
            Err(_) => LoadingResult::new_failure(LoadingError::no_package(
                184,
                self.path.to_string_lossy().to_string(),
            )),
        }) {
            for entry in paths {
                match entry {
                    Ok(path) => {
                        if let Ok(path) = path.strip_prefix(&self.path) {
                            result.merge_degrade_failure(self.insure_content(path));
                        } else {
                            result.merge_degrade_failure::<()>(LoadingResult::new_failure(
                                LoadingError::no_package(
                                    185,
                                    self.path.to_string_lossy().to_string(),
                                ),
                            ));
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        result
    }

    fn insure_loading(loader: &Loader, identifiers: Vec<Identifier>) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for identifier in identifiers {
            result.merge_degrade_failure(loader.get_with_load(&identifier));
        }

        result
    }

    fn designation(identifier: &Identifier) -> PathBuf {
        PathBuf::from(format!(
            "{}.mel",
            identifier
                .path()
                .clone()
                .into_iter()
                .skip(1)
                .collect::<Vec<_>>()
                .join("/")
        ))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn contents(&self) -> RwLockReadGuard<HashMap<PathBuf, Content>> {
        self.contents.read().unwrap()
    }
}

impl PackageTrait for FsPackage {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &Version {
        &self.version
    }

    fn requirements(&self) -> &Vec<PackageRequirement> {
        &self.requirements
    }

    fn main(&self) -> &Option<Identifier> {
        &self.main
    }

    fn embedded_collection(&self, _loader: &Loader) -> LoadingResult<Collection> {
        LoadingResult::new_success(Collection::new())
    }

    fn full_collection(&self, loader: &Loader) -> LoadingResult<Collection> {
        let mut collection = Collection::new();
        let mut result = LoadingResult::new_success(());
        if let Some(identifiers) = result.merge_degrade_failure(self.all_identifiers(loader)) {
            for identifier in identifiers {
                if collection.get(&identifier).is_none() {
                    if let Some(specific_collection) =
                        result.merge_degrade_failure(self.element(loader, &identifier))
                    {
                        for identifier in &specific_collection.identifiers() {
                            collection.insert(specific_collection.get(identifier).unwrap().clone());
                        }
                    }
                }
            }
        }

        result.and(LoadingResult::new_success(collection))
    }

    fn all_identifiers(&self, _loader: &Loader) -> LoadingResult<Vec<Identifier>> {
        let mut results = LoadingResult::new_success(Vec::new());

        results.merge(self.all_contents());
        if results.is_failure() {
            return results;
        }

        let mut identifiers = Vec::new();
        self.contents
            .read()
            .unwrap()
            .iter()
            .for_each(|(_, content)| identifiers.extend(content.provide()));

        LoadingResult::new_success(identifiers)
    }

    fn element(&self, loader: &Loader, identifier: &Identifier) -> LoadingResult<Collection> {
        let mut result = LoadingResult::new_success(Collection::new());
        let designation = Self::designation(identifier);
        if let None = result.merge_degrade_failure(self.insure_content(&designation)) {
            return result;
        }

        if let Some(content) = self.contents.read().unwrap().get(&designation) {
            if let Ok(_guard) = content.try_lock() {
                let needs = content.require();
                result.merge_degrade_failure(Self::insure_loading(loader, needs));

                let mut collection = loader.collection().clone();
                result.merge_degrade_failure(
                    content
                        .insert_descriptors(&mut collection)
                        .convert_failure_errors(|err| {
                            LoadingError::content_error(172, Arc::new(err))
                        }),
                );

                result = result.and_degrade_failure(LoadingResult::new_success(collection));
            } else {
                result.merge_degrade_failure::<()>(LoadingResult::new_failure(
                    LoadingError::circular_reference(173, identifier.clone()),
                ));
            }
        } else {
            result.merge_degrade_failure::<()>(LoadingResult::new_failure(
                LoadingError::not_found(174, identifier.to_string()),
            ));
        }

        result
    }

    fn make_building(&self, collection: &Arc<Collection>) -> LoadingResult<()> {
        let contents = self.contents.read().unwrap();
        let mut result = LoadingResult::new_success(());
        for (_, content) in contents.iter() {
            result.merge_degrade_failure(
                content
                    .make_design(collection)
                    .convert_failure_errors(|err| LoadingError::content_error(175, Arc::new(err))),
            );
        }

        result
    }
}
