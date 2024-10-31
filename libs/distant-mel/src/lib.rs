#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub mod api;

use api::*;
use core::str::FromStr;
use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_model, mel_package, mel_treatment};
use net_mel::ip::*;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Weak},
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
    initialize initialize
)]
pub struct DistantEngine {
    model: Weak<DistantEngineModel>,
    client: RwLock<Option<Arc<Client>>>,
}

impl DistantEngine {
    fn new(model: Weak<DistantEngineModel>) -> Self {
        Self {
            model,
            client: RwLock::new(None),
        }
    }

    pub fn initialize(&self) {
        let model = self.model.upgrade().unwrap();

        let address = model.get_address();
        let key = model.get_key();

        let config = TlsConfig::default().with_tcp_config(ClientConfig::new().with_nodelay(true));

        let client = Client::new(config)
            .with_base(address)
            .with_default_pool()
            .with_default_header(KnownHeaderName::UserAgent, USER_AGENT)
            .with_default_header(KnownHeaderName::Authorization, format!("Bearer {key}"));

        self.client.write().unwrap().replace(Arc::new(client));
    }

    pub async fn start(&self, request: Request) -> Result<Response, String> {
        let client = Arc::clone(self.client.read().unwrap().as_ref().unwrap());

        let connection = client
            .post("/execution/job/start")
            .with_header(KnownHeaderName::ContentType, "application/json")
            .with_body(serde_json::to_string(&request).unwrap())
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

#[mel_treatment(
    model distant_engine DistantEngine
    input trigger Block<void>
    output access Block<Access>
    output failure Block<Vec<string>>
)]
pub async fn distant(max_duration: u32, memory: u32, cpu: u32, storage: u32) {
    let model = DistantEngineModel::into(distant_engine);
    let distant = model.inner();

    let key = Uuid::new_v4();
    let start = Request {
        edition: "scratch".to_string(),
        max_duration,
        memory,
        cpu,
        mode: ModeRequest::Distribute { key: key.clone() },
        config: None,
        id: None,
        client_id: None,
        version: env!("CARGO_PKG_VERSION").to_string(),
        storage,
        arch: None,
        volumes: vec![],
        containers: vec![],
    };

    match distant.start(start).await {
        Ok(distrib) => match distrib {
            Response::Started(Some(access_info)) => {
                let _ = access
                    .send_one(Value::Data(Arc::new(Access(CommonAccess {
                        addresses: access_info.addresses,
                        port: access_info.port,
                        remote_key: access_info.key,
                        self_key: key,
                    }))))
                    .await;
            }
            Response::Started(None) => {}
            Response::Error(errs) => {
                let _ = failure.send_one(errs.into()).await;
            }
        },

        Err(err) => {
            let _ = failure.send_one(vec![err].into()).await;
        }
    }
}

mel_package!();
