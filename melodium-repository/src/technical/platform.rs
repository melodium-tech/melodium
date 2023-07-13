use super::Element;
use core::fmt::Display;
pub use platforms::Platform;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformAvailability {
    #[serde(
        serialize_with = "serialize_platform",
        deserialize_with = "deserialize_platform"
    )]
    pub platform: Platform,
    pub availability: HashMap<Availability, Element>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Mock,
    Real,
}

impl Display for Availability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Availability::Mock => write!(f, "mock"),
            Availability::Real => write!(f, "real"),
        }
    }
}

pub(crate) fn serialize_platform<S>(platform: &Platform, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(platform.target_triple)
}

pub(crate) fn deserialize_platform<'a, D>(d: D) -> Result<Platform, D::Error>
where
    D: Deserializer<'a>,
{
    let triple = Deserialize::deserialize(d)?;
    Platform::find(triple)
        .map(|p| p.clone())
        .ok_or_else(|| serde::de::Error::custom(format!("'{triple}' is not a recognized platform")))
}
