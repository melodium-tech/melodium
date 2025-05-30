use crate::access::*;
use crate::api;
use crate::resources::arch::*;
use crate::resources::*;
use async_std::process::Child;
use core::time::Duration;
use generic_async_http_client::Request;
use melodium_core::*;
use melodium_macro::{mel_model, mel_treatment};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Weak},
};
use uuid::Uuid;

#[derive(Debug)]
#[mel_model(
    param location string "api"
    param api_url Option<string> none
    param api_token Option<string> none
    initialize initialize
)]
pub struct DistantEngine {
    model: Weak<DistantEngineModel>,
    location: RwLock<Option<String>>,
    api_url: RwLock<Option<String>>,
    api_token: RwLock<Option<String>>,
    child: RwLock<Option<Arc<Child>>>,
}

impl DistantEngine {
    fn new(model: Weak<DistantEngineModel>) -> Self {
        Self {
            model,
            location: RwLock::new(None),
            api_url: RwLock::new(None),
            api_token: RwLock::new(None),
            child: RwLock::new(None),
        }
    }

    pub fn initialize(&self) {
        let model = self.model.upgrade().unwrap();

        let location = model.get_location();
        let api_url = model.get_api_url();
        let api_token = model.get_api_token();

        self.location.write().unwrap().replace(location);
        if let Some(api_url) = api_url {
            self.api_url.write().unwrap().replace(api_url);
        }
        if let Some(api_token) = api_token {
            self.api_token.write().unwrap().replace(api_token);
        }
    }

    pub async fn start(&self, request: api::Request) -> Result<api::DistributionResponse, String> {
        let location = self.location.read().unwrap().clone();
        match location.as_ref().map(|loc| loc.as_str()) {
            Some("api") => self.distrib_api(request).await,
            Some("compose") => match crate::compose::compose(request).await {
                Ok((access, child)) => {
                    self.child.write().unwrap().replace(Arc::new(child));
                    Ok(api::DistributionResponse::Started(Some(access)))
                }
                Err(err) => Ok(api::DistributionResponse::Error(err)),
            },
            Some(oth) => Err(format!(
                "\"{oth}\" is not a recognized distant execution location"
            )),
            None => Err("No location set".to_string()),
        }
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    async fn distrib_api(
        &self,
        request: api::Request,
    ) -> Result<api::DistributionResponse, String> {
        let (api_url, api_token) = (
            self.api_url.read().unwrap().clone(),
            self.api_token.read().unwrap().clone(),
        );
        if let (Some(api_url), Some(api_token)) = (api_url, api_token) {
            match Request::post(&format!("{api_url}/execution/job/start"))
                .add_header("User-Agent", crate::USER_AGENT)
                .map_err(|err| err.to_string())?
                .add_header("Authorization", format!("Bearer {api_token}").as_bytes())
                .map_err(|err| err.to_string())?
                .add_header("Content-Type", "application/json")
                .map_err(|err| err.to_string())?
                .body(serde_json::to_string(&request).unwrap())
                .map_err(|err| err.to_string())?
                .exec()
                .await
            {
                Ok(mut response) => {
                    if response.status_code() == 200 {
                        match response.json::<api::Response>().await {
                            Ok(response) => match response {
                                api::Response::Ok(id) => {
                                    async_std::task::sleep(Duration::from_secs(1)).await;
                                    loop {
                                        match Request::get(&format!(
                                            "{api_url}/execution/job/{id}/access"
                                        ))
                                        .add_header("User-Agent", crate::USER_AGENT)
                                        .map_err(|err| err.to_string())?
                                        .add_header(
                                            "Authorization",
                                            format!("Bearer {api_token}").as_bytes(),
                                        )
                                        .map_err(|err| err.to_string())?
                                        .exec()
                                        .await
                                        {
                                            Ok(mut response) => match response.status_code() {
                                                202 => {
                                                    async_std::task::sleep(Duration::from_secs(5))
                                                        .await
                                                }
                                                200 => match response.json().await {
                                                    Ok(distribution) => return Ok(distribution),
                                                    Err(error) => {
                                                        return Err(Self::manage_error(error).await)
                                                    }
                                                },
                                                code => {
                                                    return Err(format!(
                                                        "API {code} response: {response}",
                                                        response = match response.text().await {
                                                            Ok(response) => response,
                                                            Err(error) =>
                                                                Box::pin(Self::manage_error(error))
                                                                    .await,
                                                        }
                                                    ))
                                                }
                                            },
                                            Err(error) => {
                                                return Err(Self::manage_error(error).await)
                                            }
                                        }
                                    }
                                }
                                api::Response::Error(errs) => {
                                    Ok(api::DistributionResponse::Error(errs))
                                }
                            },
                            Err(error) => Err(Self::manage_error(error).await),
                        }
                    } else {
                        match response.text().await {
                            Ok(body) => Err(format!(
                                "Server {} response: {body}",
                                response.status_code()
                            )),
                            Err(error) => Err(Self::manage_error(error).await),
                        }
                    }
                }
                Err(error) => Err(Self::manage_error(error).await),
            }
        } else {
            Err("API address and token missing".into())
        }
    }

    async fn manage_error(error: generic_async_http_client::Error) -> String {
        match error {
            generic_async_http_client::Error::Io(error) => error.to_string(),
            generic_async_http_client::Error::HTTPServerErr(code, mut response) => format!(
                "API {code} error: {response}",
                response = match response.text().await {
                    Ok(text) =>
                        if text.is_empty() {
                            response.status().to_string()
                        } else {
                            format!("{}: {}", response.status(), text)
                        },
                    Err(error) => Box::pin(Self::manage_error(error)).await,
                }
            ),
            generic_async_http_client::Error::HTTPClientErr(code, mut response) => format!(
                "API {code} error: {response}",
                response = match response.text().await {
                    Ok(text) =>
                        if text.is_empty() {
                            response.status().to_string()
                        } else {
                            format!("{}: {}", response.status(), text)
                        },
                    Err(error) => Box::pin(Self::manage_error(error)).await,
                }
            ),
            generic_async_http_client::Error::Other(error) => error.to_string(),
        }
    }
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
