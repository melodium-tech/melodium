use crate::package::package::PackageTrait;
use crate::Loader;
use crate::{content::Content, PackageInfo};
use core::iter::FromIterator;
use melodium_common::descriptor::{
    Collection, Identifier, LoadingError, LoadingResult, PackageRequirement, VersionReq,
};
use semver::Version;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct RawPackage {
    name: String,
    version: Version,
    requirements: Vec<PackageRequirement>,
    entrypoints: HashMap<String, Identifier>,
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
                        name.map(|name| PackageRequirement::new(&name, &VersionReq::STAR)).unwrap_or_else(|| PackageRequirement::new("[raw package]", &VersionReq::STAR)),
                    ));
                }
            } else if let Some(internal_version) = line.strip_prefix("#! version = ") {
                if version.is_none() {
                    version = Some(match Version::parse(internal_version) {
                        Ok(val) => val,
                        Err(_) => {
                            return LoadingResult::new_failure(LoadingError::no_package(
                                188,
                                name.map(|name| PackageRequirement::new(&name, &VersionReq::STAR)).unwrap_or_else(|| PackageRequirement::new("[raw package]", &VersionReq::STAR)),
                            ))
                        }
                    });
                } else {
                    return LoadingResult::new_failure(LoadingError::no_package(
                        187,
                        name.map(|name| PackageRequirement::new(&name, &VersionReq::STAR)).unwrap_or_else(|| PackageRequirement::new("[raw package]", &VersionReq::STAR)),
                    ));
                }
            } else if let Some(internal_requirements) = line.strip_prefix("#! require = ") {
                for internal_requirement in internal_requirements.split(char::is_whitespace) {
                    let parts = internal_requirement.split(':').collect::<Vec<_>>();
                    if parts.len() == 2 {
                        if let Ok(version_requirement) = VersionReq::parse(parts[1]) {
                            requirements.push(PackageRequirement {
                                package: parts[0].to_string(),
                                version_requirement,
                            })
                        } else {
                            return LoadingResult::new_failure(LoadingError::wrong_configuration(
                                209,
                                "[raw package]".to_string(),
                            ));
                        }
                    } else {
                        return LoadingResult::new_failure(LoadingError::wrong_configuration(
                            208,
                            "[raw package]".to_string(),
                        ));
                    }
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
                    let expected_main = Identifier::new(vec![name.clone()], "main");
                    LoadingResult::new_success(Self {
                        name: name.clone(),
                        version: version.clone(),
                        requirements,
                        entrypoints: if content.provide().contains(&expected_main) {
                            HashMap::from_iter([("main".to_string(), expected_main.with_version(&version))])
                        } else {
                            HashMap::new()
                        },
                        content,
                    })
                })
        } else {
            LoadingResult::new_failure(LoadingError::no_package(
                189,
                name.map(|name| PackageRequirement::new(&name, &VersionReq::STAR)).unwrap_or_else(|| PackageRequirement::new("[raw package]", &VersionReq::STAR)),
            ))
        }
    }
}

impl PackageInfo for RawPackage {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &Version {
        &self.version
    }

    fn requirements(&self) -> &Vec<PackageRequirement> {
        &self.requirements
    }

    fn entrypoints(&self) -> &HashMap<String, Identifier> {
        &self.entrypoints
    }
}

impl PackageTrait for RawPackage {
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
        LoadingResult::new_success(self.content.provide().into_iter().map(|id| id.with_version(&self.version)).collect())
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
