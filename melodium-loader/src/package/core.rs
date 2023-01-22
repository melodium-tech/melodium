
use crate::package::package::Package;
use crate::content::Content;
use melodium_common::descriptor::{Collection, Identifier, Loader, LoadingError, Package as CommonPackage};
use semver::Version;
use std::collections::HashMap;

pub struct CorePackage {
    package: Box<dyn CommonPackage>,
    contents: HashMap<String, Content>,
}

impl CorePackage {
    pub fn new(package: Box<dyn CommonPackage>) -> Self {
        Self {
            package,
            contents: HashMap::new(),
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
