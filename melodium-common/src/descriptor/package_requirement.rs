use crate::descriptor::VersionReq;

#[derive(Clone, Debug)]
pub struct PackageRequirement {
    pub package: String,
    pub version_requirement: VersionReq,
}

impl PackageRequirement {
    pub fn new(name: &str, version_requirement: &VersionReq) -> Self {
        PackageRequirement {
            package: name.to_string(),
            version_requirement: version_requirement.clone()
        }
    }
}