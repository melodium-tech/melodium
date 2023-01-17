
use crate::package::package::Package;
use melodium_common::descriptor::{Package as CommonPackage};
use semver::Version;
pub struct CorePackage {
    package: Box<dyn CommonPackage>,
}

impl Package for CommonPackage {
    fn name(&self) -> &str {
        todo!()
    }

    fn version(&self) -> &Version {
        todo!()
    }

    fn collection(&self, loader: &dyn melodium_common::descriptor::Loader) -> Result<melodium_common::descriptor::Collection, melodium_common::descriptor::LoadingError> {
        todo!()
    }

    fn all_identifiers(&self) -> Vec<melodium_common::descriptor::Identifier> {
        todo!()
    }

    fn element(&self, loader: &dyn melodium_common::descriptor::Loader, identifier: &melodium_common::descriptor::Identifier) -> Result<melodium_common::descriptor::Collection, melodium_common::descriptor::LoadingError> {
        todo!()
    }
}
