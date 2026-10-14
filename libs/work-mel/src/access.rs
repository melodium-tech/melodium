use melodium_core::common::{
    descriptor::DataType,
    executive::{Secret as ExecutiveSecret, SecretOrigin, SecretPolicy, Value},
};
use melodium_core::*;
use melodium_macro::{mel_data, mel_function};
use net_mel::ip::*;
use std::net::IpAddr;
use uuid::Uuid;

/// Network access credentials for connecting to a distant Mélodium worker.
///
/// The authentication keys are secrets, revealed by `distrib::start` when connecting:
/// they appear neither in logs nor in debug events.
#[mel_data]
#[derive(Debug, Serialize)]
pub struct Access {
    /// IP addresses the worker can be reached on.
    pub addresses: Vec<IpAddr>,
    /// TCP port the worker listens on.
    pub port: u16,
    /// Key sent to the worker, the UUID it expects to receive, as a `Secret<string>`.
    pub remote_key: ExecutiveSecret,
    /// Key the worker sends back, the UUID it identifies itself with, as a `Secret<string>`.
    pub self_key: ExecutiveSecret,
    /// Whether the connection is plain TCP instead of TLS.
    pub disable_tls: bool,
    /// Whether plain TCP may be used with addresses other than loopback ones.
    pub allow_plain_tcp: bool,
}

impl Access {
    /// Gives an `Access` with keys generated or received at runtime, kept as inline secrets
    /// named `remote_key` and `self_key`.
    pub fn with_keys(
        addresses: Vec<IpAddr>,
        port: u16,
        remote_key: Uuid,
        self_key: Uuid,
        disable_tls: bool,
        allow_plain_tcp: bool,
    ) -> Self {
        let key = |name: &str, key: Uuid| {
            ExecutiveSecret::new(
                name.to_string(),
                DataType::String,
                SecretPolicy::default(),
                SecretOrigin::Inline(Value::String(key.to_string())),
            )
            .expect("a string value is a valid inline secret")
        };
        Self {
            addresses,
            port,
            remote_key: key("remote_key", remote_key),
            self_key: key("self_key", self_key),
            disable_tls,
            allow_plain_tcp,
        }
    }
}

/// Build an `Access` value from explicit connection parameters.
///
/// - `ip`: list of IP addresses the worker can be reached on.
/// - `port`: TCP port the worker listens on.
/// - `remote_key`: UUID the worker expects to receive (`melodium dist --recv-key-file`),
///   such as `"env:MELODIUM_SECRET_DIST_SEND_KEY"`.
/// - `self_key`: UUID the worker sends back (`melodium dist --send-key-file`), such as
///   `"env:MELODIUM_SECRET_DIST_RECV_KEY"`.
///
/// The keys are revealed when connecting, where malformed UUIDs make `distrib::start` fail.
#[mel_function]
pub fn new_access(
    ip: Vec<Ip>,
    port: u16,
    remote_key: Secret<string>,
    self_key: Secret<string>,
) -> Access {
    Access {
        addresses: ip.into_iter().map(|ip| ip.0).collect(),
        port,
        remote_key,
        self_key,
        disable_tls: false,
        allow_plain_tcp: false,
    }
}

/// Build an `Access` value for a worker listening without TLS (`melodium dist --disable-tls`).
///
/// Everything is sent readable, authentication keys included, so only loopback addresses
/// are accepted. Secrets cannot be sent by value over such a connection.
///
/// - `ip`: list of IP addresses the worker can be reached on, loopback ones only.
/// - `port`: TCP port the worker listens on.
/// - `remote_key`: UUID the worker expects to receive (`melodium dist --recv-key-file`).
/// - `self_key`: UUID the worker sends back (`melodium dist --send-key-file`).
///
/// The keys are revealed when connecting, where malformed UUIDs make `distrib::start` fail.
#[mel_function]
pub fn new_plain_access(
    ip: Vec<Ip>,
    port: u16,
    remote_key: Secret<string>,
    self_key: Secret<string>,
) -> Access {
    Access {
        addresses: ip.into_iter().map(|ip| ip.0).collect(),
        port,
        remote_key,
        self_key,
        disable_tls: true,
        allow_plain_tcp: false,
    }
}
