use core::{
    fmt::{Display, Formatter},
    str::FromStr,
};
use melodium_common::descriptor::{Identifier as CommonIdentifier, Version};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Identifier {
    pub version: Option<String>,
    pub path: Vec<String>,
    pub name: String,
}

impl Identifier {
    pub fn root(&self) -> &str {
        self.path.first().map(|s| s.as_str()).unwrap_or("")
    }

    pub fn with_version(self, version: String) -> Self {
        Self {
            version: Some(version),
            path: self.path,
            name: self.name,
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut string = self.path.join("/");

        string = string + "::" + &self.name;

        write!(f, "{}", string)
    }
}

impl From<&CommonIdentifier> for Identifier {
    fn from(value: &CommonIdentifier) -> Self {
        Self {
            version: value.version().map(|v| v.to_string()),
            name: value.name().to_string(),
            path: value.path().clone(),
        }
    }
}

impl FromStr for Identifier {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match CommonIdentifier::from_str(value) {
            Ok(id) => Ok(Self::from(&id)),
            Err(_) => Err(()),
        }
    }
}

impl TryInto<CommonIdentifier> for &Identifier {
    type Error = Self;

    fn try_into(self) -> Result<CommonIdentifier, Self::Error> {
        if let Some(version) = &self.version {
            match Version::parse(version) {
                Ok(version) => Ok(CommonIdentifier::new_versionned(
                    &version,
                    self.path.clone(),
                    &self.name,
                )),
                Err(_) => Err(self),
            }
        } else {
            Ok(CommonIdentifier::new(self.path.clone(), &self.name))
        }
    }
}

impl TryInto<CommonIdentifier> for Identifier {
    type Error = Self;

    fn try_into(self) -> Result<CommonIdentifier, Self::Error> {
        if let Some(version) = &self.version {
            match Version::parse(version) {
                Ok(version) => Ok(CommonIdentifier::new_versionned(
                    &version, self.path, &self.name,
                )),
                Err(_) => Err(self),
            }
        } else {
            Ok(CommonIdentifier::new(self.path, &self.name))
        }
    }
}
