use crate::content::{Content, ContentError};
use crate::package::package::Package;
use crate::Loader;
use melodium_common::descriptor::{Collection, Identifier, LoadingError};
use semver::Version;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct RawPackage {
    name: String,
    version: Version,
    requirements: Vec<String>,
    content: Content,
    errors: RwLock<Vec<ContentError>>,
}

impl RawPackage {
    pub fn new(data: &str) -> Result<Self, LoadingError> {
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
                    return Err(LoadingError::ContentError);
                }
            } else if let Some(internal_version) = line.strip_prefix("#! version = ") {
                if version.is_none() {
                    version = Some(
                        Version::parse(internal_version).map_err(|_| LoadingError::ContentError)?,
                    );
                } else {
                    return Err(LoadingError::ContentError);
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

        if let (Some(name), Some(version)) = (name, version) {
            let content =
                Content::new(&name, data.as_bytes()).map_err(|_| LoadingError::ContentError)?;

            Ok(Self {
                name,
                version,
                requirements,
                content,
                errors: RwLock::new(Vec::new()),
            })
        } else {
            Err(LoadingError::ContentError)
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

    fn embedded_collection(&self, _: &Loader) -> Result<Collection, LoadingError> {
        Ok(Collection::new())
    }

    fn full_collection(&self, loader: &Loader) -> Result<Collection, LoadingError> {
        let mut collection = Collection::new();

        for need in self.content.require() {
            collection.insert(loader.get_with_load(&need)?);
        }

        match self.content.insert_descriptors(&mut collection) {
            Ok(()) => Ok(collection),
            Err(error) => {
                self.errors.write().unwrap().push(error);
                Err(LoadingError::ContentError)
            }
        }
    }

    fn all_identifiers(&self, _: &Loader) -> Result<Vec<Identifier>, LoadingError> {
        Ok(self.content.provide())
    }

    fn element(
        &self,
        loader: &Loader,
        _identifier: &Identifier,
    ) -> Result<Collection, LoadingError> {
        self.full_collection(loader)
    }

    fn make_building(&self, collection: &Arc<Collection>) -> Result<(), LoadingError> {
        if let Err(error) = self.content.make_design(collection) {
            self.errors.write().unwrap().push(error);
            return Err(LoadingError::ContentError);
        }

        Ok(())
    }

    fn errors(&self) -> Vec<ContentError> {
        self.errors.read().unwrap().clone()
    }
}
