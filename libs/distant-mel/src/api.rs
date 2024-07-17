use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Access {
    pub address_v4: Ipv4Addr,
    pub address_v6: Ipv6Addr,
    pub port: u16,
    pub key: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Start {
    pub key: Uuid,
    pub edition: String,
    pub max_duration: u32,
    pub memory: u64,
    pub cpu: u16,
    pub mode: StartMode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StartMode {
    Entry {},
    Distribution {},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Distributed {
    Success(Access),
    Failure(String),
}
