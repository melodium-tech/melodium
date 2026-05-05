use crate::api;
use core::str::FromStr;
use melodium_core::*;
use melodium_macro::{mel_data, mel_function};
use net_mel::ip::*;
use uuid::Uuid;

/// Network access credentials for connecting to a distant Mélodium worker.
#[mel_data]
#[derive(Debug, Serialize)]
pub struct Access(pub api::CommonAccess);

/// Build an `Access` value from explicit connection parameters.
///
/// - `ip`: list of IP addresses the worker can be reached on.
/// - `port`: TCP port the worker listens on.
/// - `remote_key`: UUID identifying the remote side (as a string).
/// - `self_key`: UUID identifying the local side (as a string).
///
/// ⚠️ Malformed UUID strings are silently replaced with the nil UUID.
#[mel_function]
pub fn new_access(ip: Vec<Ip>, port: u16, remote_key: string, self_key: string) -> Access {
    Access(api::CommonAccess {
        addresses: ip.into_iter().map(|ip| ip.0).collect(),
        port,
        remote_key: Uuid::from_str(&remote_key).unwrap_or_default(),
        self_key: Uuid::from_str(&self_key).unwrap_or_default(),
        disable_tls: false,
    })
}
