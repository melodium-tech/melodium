use crate::descriptor::VersionReq;

#[derive(Clone, Debug)]
pub struct PackageRequirement {
    pub package: String,
    pub version_requirement: VersionReq,
}
