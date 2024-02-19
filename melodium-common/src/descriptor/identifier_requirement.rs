
use core::fmt::{Display, Formatter};
use crate::descriptor::{PackageRequirement, VersionReq};

#[derive(Clone, Debug)]
pub struct IdentifierRequirement {
    version_requirement: VersionReq,
    path: Vec<String>,
    name: String,
}

impl IdentifierRequirement {
    pub fn new(version_requirement: VersionReq, path: Vec<String>, name: &str) -> Self {
        if path.is_empty() {
            panic!("Identifier path cannot be empty.")
        }

        Self {
            version_requirement,
            path,
            name: name.to_string(),
        }
    }

    pub fn version_requirement(&self) -> &VersionReq {
        &self.version_requirement
    }

    pub fn root(&self) -> &String {
        &self.path.first().unwrap()
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn package_requirement(&self) -> PackageRequirement {
        PackageRequirement::new(self.root(), &self.version_requirement)
    }
}

impl Display for IdentifierRequirement {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut string = self.path.join("/");

        string = string + "::" + &self.name;

        write!(f, "{} ({})", string, self.version_requirement)
    }
}