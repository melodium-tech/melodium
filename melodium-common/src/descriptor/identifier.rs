use super::Version;
use core::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    result::Result,
    str::FromStr,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Identifier {
    version: Option<Version>,
    path: Vec<String>,
    name: String,
}

impl Identifier {
    pub fn new(path: Vec<String>, name: &str) -> Self {
        if path.is_empty() {
            panic!("Identifier path cannot be empty.")
        }

        Self {
            version: None,
            path,
            name: name.to_string(),
        }
    }

    pub fn new_versionned(version: &Version, path: Vec<String>, name: &str) -> Self {
        if path.is_empty() {
            panic!("Identifier path cannot be empty.")
        }

        Self {
            version: Some(version.clone()),
            path,
            name: name.to_string(),
        }
    }

    pub fn new_optionally_versionned(
        version: Option<&Version>,
        path: Vec<String>,
        name: &str,
    ) -> Self {
        if path.is_empty() {
            panic!("Identifier path cannot be empty.")
        }

        Self {
            version: version.cloned(),
            path,
            name: name.to_string(),
        }
    }

    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
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

    pub fn with_version(&self, version: &Version) -> Self {
        Self {
            version: Some(version.clone()),
            path: self.path.clone(),
            name: self.name.clone(),
        }
    }

    pub fn with_optionnal_version(&self, version: Option<&Version>) -> Self {
        Self {
            version: version.cloned(),
            path: self.path.clone(),
            name: self.name.clone(),
        }
    }

    pub fn without_version(&self) -> Self {
        Self {
            version: None,
            path: self.path.clone(),
            name: self.name.clone(),
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut string = self.path.join("/");

        string = string + "::" + &self.name;

        if let Some(version) = &self.version {
            write!(f, "{} ({})", string, version)
        } else {
            write!(f, "{}", string)
        }
    }
}

impl FromStr for Identifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.to_string().partial_cmp(&other.to_string())
    }
}

impl TryFrom<&str> for Identifier {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let full = value.split("::").collect::<Vec<_>>();
        if full.len() == 2 {
            let path = full[0];
            let name = full[1];

            let path = path.split('/').map(|s| s.to_string()).collect::<Vec<_>>();
            if path.len() >= 1 {
                Ok(Self {
                    version: None,
                    path,
                    name: name.to_string(),
                })
            } else {
                Err(value.to_string())
            }
        } else {
            Err(value.to_string())
        }
    }
}

impl TryFrom<&String> for Identifier {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
