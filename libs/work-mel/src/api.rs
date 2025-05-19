use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, net::IpAddr};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommonAccess {
    pub addresses: Vec<IpAddr>,
    pub port: u16,
    pub remote_key: Uuid,
    pub self_key: Uuid,
    #[serde(skip)] // Default to false
    pub disable_tls: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Access {
    pub id: Uuid,
    pub addresses: Vec<IpAddr>,
    pub port: u16,
    pub key: Uuid,
    #[serde(skip)] // Default to false
    pub disable_tls: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub config: Option<RequestConfig>,
    pub id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub edition: String,
    pub version: String,
    pub mode: ModeRequest,
    pub max_duration: u32,
    pub memory: u32,
    pub cpu: u32,
    pub storage: u32,
    pub arch: Option<Arch>,
    pub volumes: Vec<Volume>,
    pub containers: Vec<Container>,
    pub service_containers: Vec<ServiceContainer>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestConfig {
    pub cluster_mode_preference: Option<Vec<ServiceMode>>,
    pub cluster_mode_requirement: Option<Vec<ServiceMode>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceMode {
    Shared,
    Dedicated,
    SelfManaged,
    #[serde(other, deserialize_with = "ignore_contents")]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeRequest {
    Direct { entrypoint: String, project: () },
    Distribute { key: Uuid },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Arch {
    Amd64,
    Arm64,
    #[serde(other, deserialize_with = "ignore_contents")]
    Unknown,
}
impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::Amd64 => write!(f, "amd64"),
            Arch::Arm64 => write!(f, "arm64"),
            _ => write!(f, "unknown"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Volume {
    pub name: String,
    pub storage: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub pull_secret: String,
    pub memory: u32,
    pub cpu: u32,
    pub storage: u32,
    pub arch: Arch,
    pub mounts: Vec<VolumeMount>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceContainer {
    pub name: String,
    pub image: String,
    pub pull_secret: String,
    pub memory: u32,
    pub cpu: u32,
    pub storage: u32,
    pub arch: Arch,
    pub mounts: Vec<VolumeMount>,
    pub env: HashMap<String, String>,
    pub command: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    pub mount_point: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    Ok(Uuid),
    Error(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionResponse {
    Started(Option<Access>),
    Error(Vec<String>),
}

fn ignore_contents<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    // Ignore any content at this part of the json structure
    let _ = deserializer.deserialize_ignored_any(serde::de::IgnoredAny);

    // Return unit as our 'Unknown' variant has no args
    Ok(())
}
