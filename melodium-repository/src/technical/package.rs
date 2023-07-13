use super::Type;
use melodium_common::descriptor::{PackageRequirement, Version, VersionReq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Version,
    #[serde(
        serialize_with = "serialize_vec_requirement",
        deserialize_with = "deserialize_vec_requirement"
    )]
    pub requirements: Vec<PackageRequirement>,
    pub r#type: Type,
}

impl Package {
    pub fn get_path(&self) -> PathBuf {
        let mut path = PathBuf::from(self.name.clone());
        path.push(self.version.to_string());
        path
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageRequirementDef {
    pub package: String,
    pub version_requirement: VersionReq,
}

impl From<&PackageRequirement> for PackageRequirementDef {
    fn from(value: &PackageRequirement) -> Self {
        Self {
            package: value.package.clone(),
            version_requirement: value.version_requirement.clone(),
        }
    }
}

impl Into<PackageRequirement> for PackageRequirementDef {
    fn into(self) -> PackageRequirement {
        PackageRequirement {
            package: self.package,
            version_requirement: self.version_requirement,
        }
    }
}

pub(crate) fn serialize_vec_requirement<S>(
    vec_requirement: &Vec<PackageRequirement>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let vec_requirement: Vec<PackageRequirementDef> = vec_requirement
        .iter()
        .map(PackageRequirementDef::from)
        .collect();

    vec_requirement.serialize(s)
}

pub(crate) fn deserialize_vec_requirement<'a, D>(d: D) -> Result<Vec<PackageRequirement>, D::Error>
where
    D: Deserializer<'a>,
{
    let vec_requirement: Vec<PackageRequirementDef> = Deserialize::deserialize(d)?;

    Ok(vec_requirement.into_iter().map(|pr| pr.into()).collect())
}
