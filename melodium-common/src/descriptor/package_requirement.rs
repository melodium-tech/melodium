use core::fmt::{Display, Formatter};
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

impl Display for PackageRequirement {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ({})", self.package, self.version_requirement)
    }
}