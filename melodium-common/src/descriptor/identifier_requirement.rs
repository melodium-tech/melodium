
use core::fmt::{Display, Formatter};
use crate::descriptor::{PackageRequirement, VersionReq, Identifier};

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

    pub fn new_with_identifier(version_requirement: VersionReq, identifier: &Identifier) -> Self {
        Self {
            version_requirement,
            path: identifier.path().clone(),
            name: identifier.name().to_string(),
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

    pub fn to_identifier(&self) -> Identifier {
        Identifier::new(self.path.clone(), &self.name)
    }

    pub fn package_requirement(&self) -> PackageRequirement {
        PackageRequirement::new(self.root(), &self.version_requirement)
    }
}

impl From<&Identifier> for IdentifierRequirement {
    fn from(value: &Identifier) -> Self {
        IdentifierRequirement::new(value.version().map(|v| VersionReq {
            comparators: vec![semver::Comparator {op:semver::Op::Exact, major: v.major, minor: Some(v.minor), patch: Some(v.patch), pre: v.pre.clone() }],
        }).unwrap_or_default(), value.path().clone(), value.name())
    }
}

impl Display for IdentifierRequirement {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut string = self.path.join("/");

        string = string + "::" + &self.name;

        write!(f, "{} ({})", string, self.version_requirement)
    }
}