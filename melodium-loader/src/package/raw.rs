use crate::content::Content;
use crate::package::package::Package;
use crate::Loader;
use melodium_common::descriptor::{Collection, Identifier, LoadingError, LoadingResult};
use semver::Version;
use std::sync::Arc;

#[derive(Debug)]
pub struct RawPackage {
    name: String,
    version: Version,
    requirements: Vec<String>,
    content: Content,
}

impl RawPackage {
    pub fn new(data: &str) -> LoadingResult<Self> {
        let mut name = None;
        let mut version = None;
        let mut requirements = Vec::new();
        for line in data.lines() {
            if line.starts_with("#!/") {
                continue;
            } else if let Some(internal_name) = line.strip_prefix("#! name = ") {
                if name.is_none() {
                    name = Some(internal_name.to_string());
                } else {
                    return LoadingResult::new_failure(LoadingError::no_package(
                        186,
                        name.unwrap_or_else(|| "[raw package]".to_string()),
                    ));
                }
            } else if let Some(internal_version) = line.strip_prefix("#! version = ") {
                if version.is_none() {
                    version = Some(match Version::parse(internal_version) {
                        Ok(val) => val,
                        Err(_) => {
                            return LoadingResult::new_failure(LoadingError::no_package(
                                188,
                                name.unwrap_or_else(|| "[raw package]".to_string()),
                            ))
                        }
                    });
                } else {
                    return LoadingResult::new_failure(LoadingError::no_package(
                        187,
                        name.unwrap_or_else(|| "[raw package]".to_string()),
                    ));
                }
            } else if let Some(internal_requirements) = line.strip_prefix("#! require = ") {
                for internal_requirement in internal_requirements.split(char::is_whitespace) {
                    requirements.push(internal_requirement.to_string());
                }
            } else if line.starts_with("#!") {
                continue;
            } else {
                break;
            }
        }

        if version.is_none() {
            version = Some(Version::new(0, 1, 0));
        }

        if let (Some(name), Some(version)) = (&name, version) {
            Content::new(&name, data.as_bytes())
                .convert_failure_errors(|err| LoadingError::content_error(190, Arc::new(err)))
                .and_then(|content| {
                    LoadingResult::new_success(Self {
                        name: name.clone(),
                        version,
                        requirements,
                        content,
                    })
                })
        } else {
            LoadingResult::new_failure(LoadingError::no_package(
                189,
                name.clone().unwrap_or_else(|| "[raw package]".to_string()),
            ))
        }
    }
}

impl Package for RawPackage {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &Version {
        &self.version
    }

    fn requirements(&self) -> &Vec<String> {
        &self.requirements
    }

    fn embedded_collection(&self, _: &Loader) -> LoadingResult<Collection> {
        LoadingResult::new_success(Collection::new())
    }

    fn full_collection(&self, loader: &Loader) -> LoadingResult<Collection> {
        let mut result = LoadingResult::new_success(Collection::new());
        let mut collection = Collection::new();

        for need in self.content.require() {
            if let Some(entry) = result.merge_degrade_failure(loader.get_with_load(&need)) {
                collection.insert(entry);
            }
        }

        result
            .and_then(|_| {
                self.content
                    .insert_descriptors(&mut collection)
                    .convert_failure_errors(|err| LoadingError::content_error(191, Arc::new(err)))
            })
            .and(LoadingResult::new_success(collection))
    }

    fn all_identifiers(&self, _: &Loader) -> LoadingResult<Vec<Identifier>> {
        LoadingResult::new_success(self.content.provide())
    }

    fn element(&self, loader: &Loader, _identifier: &Identifier) -> LoadingResult<Collection> {
        self.full_collection(loader)
    }

    fn make_building(&self, collection: &Arc<Collection>) -> LoadingResult<()> {
        self.content
            .make_design(collection)
            .convert_failure_errors(|err| LoadingError::content_error(192, Arc::new(err)))
    }
}
