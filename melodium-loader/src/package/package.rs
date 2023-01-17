
use melodium_common::descriptor::{Collection, Identifier, Loader, LoadingError};
use semver::Version;

pub trait Package {
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
    fn collection(&self, loader: &dyn Loader) -> Result<Collection, LoadingError>;
    fn all_identifiers(&self) -> Vec<Identifier>;
    fn element(&self, loader: &dyn Loader, identifier: &Identifier) -> Result<Collection, LoadingError>;
}
