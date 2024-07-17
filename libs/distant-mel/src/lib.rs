#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub mod api;

use api::*;
use melodium_core::*;
use melodium_macro::{mel_data, mel_model, mel_package, mel_treatment};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use trillium_async_std::ClientConfig;
use trillium_client::{Client, KnownHeaderName};
#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
use trillium_native_tls::NativeTlsConfig as TlsConfig;
#[cfg(all(not(target_env = "msvc"), not(target_vendor = "apple")))]
use trillium_rustls::RustlsConfig as TlsConfig;
use uuid::Uuid;

pub const USER_AGENT: &str = concat!("distant-mel/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
#[mel_model(
    param address string none
    param key string none
)]
pub struct DistantEngine {
    model: Weak<DistantEngineModel>,
    client: Client,
}

impl DistantEngine {
    fn new(model: Weak<DistantEngineModel>) -> Self {
        let config = TlsConfig::default().with_tcp_config(ClientConfig::new().with_nodelay(true));

        let client = Client::new(config)
            .with_default_pool()
            .with_default_header(KnownHeaderName::UserAgent, USER_AGENT);

        Self { model, client }
    }

    pub async fn start(&self, start: Start) -> Result<Distributed, String> {
        let model = self.model.upgrade().unwrap();

        let address = model.get_address();

        let connection = self
            .client
            .post(address)
            .with_header(KnownHeaderName::ContentType, "application/json")
            .with_body(serde_json::to_string(&start).unwrap())
            .await
            .map_err(|err| err.to_string())?;

        match connection.success() {
            Ok(mut connection) => serde_json::from_str(
                &connection
                    .response_body()
                    .read_string()
                    .await
                    .map_err(|err| err.to_string())?,
            )
            .map_err(|err| err.to_string()),
            Err(status) => Err(status.to_string()),
        }
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}
}

#[mel_data]
#[derive(Debug, Serialize)]
pub struct Access(pub api::Access);

#[mel_treatment(
    model distant_engine DistantEngine
    input trigger Block<void>
    output access Block<Access>
    output failure Block<string>
)]
pub async fn distant(max_duration: u32, memory: u64, cpu: u16) {
    let model = DistantEngineModel::into(distant_engine);
    let distant = model.inner();

    let start = Start {
        key: Uuid::new_v4(),
        edition: "registry.gitlab.com/melodium/melodium-private-developments:scratch".to_string(),
        max_duration,
        memory,
        cpu,
        mode: StartMode::Distribution {},
    };

    match distant.start(start).await {
        Ok(distrib) => match distrib {
            Distributed::Success(access_info) => {
                let _ = access
                    .send_one(Value::Data(Arc::new(Access(access_info))))
                    .await;
            }
            Distributed::Failure(err) => {
                let _ = failure.send_one(err.into()).await;
            }
        },

        Err(err) => {
            let _ = failure.send_one(err.into()).await;
        }
    }
}

mel_package!();
