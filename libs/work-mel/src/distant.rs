use crate::access::*;
use crate::api;
use crate::resources::arch::*;
use crate::resources::*;
use async_std::process::Child;
use core::time::Duration;
use melodium_core::*;
use melodium_macro::{mel_model, mel_treatment};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Weak},
};
use trillium_async_std::async_std;
use trillium_async_std::ClientConfig;
use trillium_client::Status;
use trillium_client::{Client, KnownHeaderName};
#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
use trillium_native_tls::NativeTlsConfig as TlsConfig;
#[cfg(all(not(target_env = "msvc"), not(target_vendor = "apple")))]
use trillium_rustls::RustlsConfig as TlsConfig;
use uuid::Uuid;

#[derive(Debug)]
#[mel_model(
    param address string none
    param key string none
    initialize initialize
)]
pub struct DistantEngine {
    model: Weak<DistantEngineModel>,
    client: RwLock<Option<Arc<Client>>>,
    child: RwLock<Option<Arc<Child>>>,
}

impl DistantEngine {
    fn new(model: Weak<DistantEngineModel>) -> Self {
        Self {
            model,
            client: RwLock::new(None),
            child: RwLock::new(None),
        }
    }

    pub fn initialize(&self) {
        let model = self.model.upgrade().unwrap();

        let address = model.get_address();
        let key = model.get_key();

        if address.as_str() == "local" {
            // Nothing to do, using containers
        } else {
            let config =
                TlsConfig::default().with_tcp_config(ClientConfig::new().with_nodelay(true));

            let client = Client::new(config)
                .with_base(address)
                .with_default_pool()
                .with_default_header(KnownHeaderName::UserAgent, crate::USER_AGENT)
                .with_default_header(KnownHeaderName::Authorization, format!("Bearer {key}"));

            self.client.write().unwrap().replace(Arc::new(client));
        }
    }

    pub async fn start(&self, request: api::Request) -> Result<api::DistributionResponse, String> {
        let client = self
            .client
            .read()
            .unwrap()
            .as_ref()
            .map(|client| Arc::clone(client));
        if let Some(client) = client {
            let connection = client
                .post("/execution/job/start")
                .with_request_header(KnownHeaderName::ContentType, "application/json")
                .with_body(serde_json::to_string(&request).unwrap())
                .await
                .map_err(|err| err.to_string())?;

            match connection.success() {
                Ok(mut connection) => {
                    match serde_json::from_str::<api::Response>(
                        &connection
                            .response_body()
                            .read_string()
                            .await
                            .map_err(|err| err.to_string())?,
                    )
                    .map_err(|err| err.to_string())?
                    {
                        api::Response::Ok(id) => {
                            async_std::task::sleep(Duration::from_secs(1)).await;
                            loop {
                                let mut connection = client
                                    .get(format!("/execution/job/{id}/access"))
                                    .await
                                    .map_err(|err| err.to_string())?;
                                match connection.status() {
                                    Some(Status::Accepted) => {
                                        async_std::task::sleep(Duration::from_secs(5)).await;
                                    }
                                    Some(Status::Ok) => {
                                        return Ok(serde_json::from_str(
                                            &connection
                                                .response_body()
                                                .read_string()
                                                .await
                                                .map_err(|err| err.to_string())?,
                                        )
                                        .map_err(|err| err.to_string())?)
                                    }
                                    Some(code) => return Err(format!("API responded {code}")),
                                    None => return Err("Invalid response from API".to_string()),
                                }
                            }
                        }
                        api::Response::Error(err) => Ok(api::DistributionResponse::Error(err)),
                    }
                }
                Err(status) => Err(status.to_string()),
            }
        } else {
            match crate::compose::compose(request).await {
                Ok((access, child)) => {
                    self.child.write().unwrap().replace(Arc::new(child));
                    Ok(api::DistributionResponse::Started(Some(access)))
                }
                Err(err) => Ok(api::DistributionResponse::Error(err)),
            }
        }
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}
}

/// Request for a distant worker.
///
/// Send a request to get a distant Mélodium worker, on which program distribution can be done.
///
/// - `access` is emitted once worker is accessible.
/// - `failed` is emitted if the worker request cannot be satisfied.
/// - `errors` stream the error messages that can occurs.
///
/// The request is based on given parameters:
///
/// - `cpu`: CPU amount requested for the worker, in millicores (`1000` means one full CPU, `500` half of it);
/// - `memory`: memory requested for the worker, in megabytes;
/// - `storage`: filesystem storage requested for the worker, in megabytes;
/// - `max_duration`: maximum duration for which the worker will be effective, in seconds;
///
/// - `arch`: hardware architecture the worker must have (should be none if nothing specific is required);
/// - `edition`: Mélodium edition the worker must rely on (see on the Mélodium Services documentation to get the full list, can be none if nothing specific is required);
///
/// - `containers`: list of containers to instanciate alongside Mélodium engine as executors;
/// - `service_containers`: list of containers to instanciate alongside Mélodium engine as services;
/// - `volumes`: list of filesystem volumes that can be shared between the Mélodium engine and containers.
///
/// It should be noted that the CPU and memory requirements for the Mélodium engine and possible containers are cumulative.
/// Also, multiple different architecture cannot be requested for the same worker, so containers in the same request all have to use the same architecture.
/// Finally, the cumuled size of all volumes must be equal or less than the Mélodium engine storage value,
/// and each container must have storage values at least equal to the sum of the volumes mounted inside them.
///
#[mel_treatment(
    model distant_engine DistantEngine
    input trigger Block<void>
    output access Block<Access>
    output failed Block<void>
    output errors Stream<string>
)]
pub async fn distant(
    max_duration: u32,
    memory: u32,
    cpu: u32,
    storage: u32,
    edition: Option<string>,
    arch: Option<Arch>,
    volumes: Vec<Volume>,
    containers: Vec<Container>,
    service_containers: Vec<ServiceContainer>,
) {
    let model = DistantEngineModel::into(distant_engine);
    let distant = model.inner();

    let key = Uuid::new_v4();
    let start = api::Request {
        edition: edition.unwrap_or_else(|| "scratch".to_string()),
        max_duration,
        memory,
        cpu,
        mode: api::ModeRequest::Distribute { key: key.clone() },
        config: None,
        id: None,
        organization_id: None,
        version: env!("CARGO_PKG_VERSION").to_string(),
        storage,
        arch: arch.map(|arch| arch.0),
        volumes: volumes.into_iter().map(|vol| vol.0.clone()).collect(),
        containers: containers.into_iter().map(|cont| cont.0.clone()).collect(),
        service_containers: service_containers
            .into_iter()
            .map(|cont| cont.0.clone())
            .collect(),
    };

    if let Ok(_) = trigger.recv_one().await {
        match distant.start(start).await {
            Ok(distrib) => match distrib {
                api::DistributionResponse::Started(Some(access_info)) => {
                    let _ = access
                        .send_one(Value::Data(Arc::new(Access(api::CommonAccess {
                            addresses: access_info.addresses,
                            port: access_info.port,
                            remote_key: access_info.key,
                            self_key: key,
                            disable_tls: access_info.disable_tls,
                        }))))
                        .await;
                }
                api::DistributionResponse::Started(None) => {}
                api::DistributionResponse::Error(errs) => {
                    let _ = failed.send_one(().into()).await;
                    let _ = errors.send_many(errs.into()).await;
                }
            },

            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = errors.send_many(vec![err].into()).await;
            }
        }
    }
}
