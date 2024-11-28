use crate::api;
use core::str::FromStr;
use melodium_core::*;
use melodium_macro::{mel_data, mel_function};
use net_mel::ip::*;
use uuid::Uuid;

#[mel_data]
#[derive(Debug, Serialize)]
pub struct Access(pub api::CommonAccess);

#[mel_function]
pub fn new_access(
    ipv4: Ipv4,
    ipv6: Ipv6,
    port: u16,
    remote_key: string,
    self_key: string,
) -> Access {
    Access(api::CommonAccess {
        addresses: vec![ipv6.0.into(), ipv4.0.into()],
        port,
        remote_key: Uuid::from_str(&remote_key).unwrap_or_default(),
        self_key: Uuid::from_str(&self_key).unwrap_or_default(),
    })
}
