use crate::descriptor::{Collection, Loader, LoadingError};
use core::fmt::Debug;
use semver::Version;
use std::collections::HashMap;

pub trait Package: Debug {
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
    fn requirements(&self) -> &Vec<&str>;
    fn collection(&self, loader: &dyn Loader) -> Result<Collection, LoadingError>;
    fn embedded(&self) -> &HashMap<&'static str, &'static [u8]>;
}
