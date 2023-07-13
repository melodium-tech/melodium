use crate::descriptor::{Collection, Loader, LoadingResult, Version, VersionReq};
use core::{any::Any, fmt::Debug};
use std::collections::HashMap;

pub trait Package: Debug + Any + Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
    fn requirements(&self) -> &Vec<PackageRequirement>;
    fn collection(&self, loader: &dyn Loader) -> LoadingResult<Collection>;
    fn embedded(&self) -> &HashMap<&'static str, &'static [u8]>;
}

#[derive(Clone, Debug)]
pub struct PackageRequirement {
    pub package: String,
    pub version_requirement: VersionReq,
}
