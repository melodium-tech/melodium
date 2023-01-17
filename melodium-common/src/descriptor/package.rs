
use crate::descriptor::{Collection, Loader, LoadingError};
use semver::Version;
use std::collections::HashMap;

pub trait Package {
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
    fn collection(&self, loader: &dyn Loader) -> Result<Collection, LoadingError>;
    fn embedded(&self) -> &HashMap<&'static str, &'static [u8]>;
}
