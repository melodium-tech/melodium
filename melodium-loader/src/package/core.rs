
use crate::package::package::Package;
use crate::content::{Content, ContentError};
use melodium_common::descriptor::{Collection, Identifier, Loader, LoadingError, Package as CommonPackage};
use semver::Version;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct CorePackage {
    package: Box<dyn CommonPackage>,
    contents: RwLock<HashMap<String, Content>>,
    errors: RwLock<Vec<ContentError>>,
}

impl CorePackage {
    pub fn new(package: Box<dyn CommonPackage>) -> Self {
        Self {
            package,
            contents: RwLock::new(HashMap::new()),
            errors: RwLock::new(Vec::new()),
        }
    }

    fn new_content(&self, loader: &dyn Loader, designation: String) -> Result<(), LoadingError> {

        match self.package.embedded().get(designation.as_str()) {
            Some(data) => {

                let result_content = Content::new(designation.as_str(), data);

                match result_content {
                    Ok(content) => {
                        self.contents.write().unwrap().insert(designation, content);
                        Ok(())
                    },
                    Err(error) => {
                        self.errors.write().unwrap().push(error);
                        Err(LoadingError::NotFound)
                    }
                }

                
            },
            None => Err(LoadingError::NotFound)
        }
    }
}

impl Package for CorePackage {
    fn name(&self) -> &str {
        self.package.name()
    }

    fn version(&self) -> &Version {
        self.package.version()
    }

    fn embedded_collection(&self, loader: &dyn Loader) -> Result<Collection, LoadingError> {
        self.package.collection(loader)
    }

    fn full_collection(&self, loader: &dyn Loader) -> Result<Collection, LoadingError> {
        self.package.collection(loader)
    }

    fn all_identifiers(&self) -> Vec<Identifier> {
        todo!()
    }

    fn element(&self, loader: &dyn Loader, identifier: &Identifier) -> Result<Collection, LoadingError> {
        todo!()
    }
}
