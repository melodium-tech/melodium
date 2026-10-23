use crate::access::*;
use crate::api;
#[cfg(feature = "real")]
use crate::http;
use crate::resources::arch::*;
use crate::resources::system::*;
use crate::resources::*;
use core::time::Duration;
use melodium_core::common::{
    descriptor::DataType,
    executive::{
        wipe_value, Secret as ExecutiveSecret, SecretError, SecretOrigin, SecretPolicy,
        SecretTransfer, SecretTransmission, SecretsAccess, Value,
    },
};
use melodium_core::*;
use melodium_macro::{mel_function, mel_model, mel_treatment};
use std::sync::{Arc, RwLock, Weak};
use uuid::Uuid;
use zeroize::Zeroizing;

#[derive(Debug)]
/// Model for requesting and connecting to a distant Mélodium worker.
///
/// `DistantEngine` sends a worker request to the Mélodium Services API or a local Compose
/// deployment, waits for the worker to become available, and provides an `Access` value for
/// establishing a distribution connection.
///
/// - `location`: where to submit the request: `"api"` (default) for Mélodium Services, or `"compose"` for a local Docker/Podman Compose deployment.
/// - `api_url`: base URL of the Mélodium Services API; defaults to the built-in endpoint.
/// - `api_token`: authentication token for the API (such as `"env:MELODIUM_SECRET_API_TOKEN"`); defaults to the `MELODIUM_API_TOKEN` environment variable when it is set.
///
/// Use the `distant` treatment to trigger a worker request.
#[mel_model(
    param location string "api"
    param api_url Option<string> none
    param api_token Option<Secret<string>> none
    secrets_access
    initialize initialize
)]
pub struct DistantEngine {
    model: Weak<DistantEngineModel>,
    location: RwLock<Option<String>>,
    api_url: RwLock<Option<String>>,
    api_token: RwLock<Option<ExecutiveSecret>>,
}

impl DistantEngine {
    fn new(model: Weak<DistantEngineModel>) -> Self {
        Self {
            model,
            location: RwLock::new(None),
            api_url: RwLock::new(None),
            api_token: RwLock::new(None),
        }
    }

    pub fn initialize(&self) {
        let model = self.model.upgrade().unwrap();

        let location = model.get_location();
        let api_url = model
            .get_api_url()
            .or_else(|| Some(crate::API_URL.to_string()));
        // `MELODIUM_API_TOKEN` configures Mélodium itself: read here, and not through an
        // `env:` locator, as programs only get the variables prefixed `MELODIUM_SECRET_`.
        let api_token = model.get_api_token().or_else(|| {
            std::env::var("MELODIUM_API_TOKEN").ok().and_then(|token| {
                ExecutiveSecret::new(
                    "MELODIUM_API_TOKEN".to_string(),
                    DataType::String,
                    SecretPolicy::default(),
                    SecretOrigin::Inline(Value::String(token)),
                )
                .ok()
            })
        });

        self.location.write().unwrap().replace(location);
        if let Some(api_url) = api_url {
            self.api_url.write().unwrap().replace(api_url);
        }
        if let Some(api_token) = api_token {
            self.api_token.write().unwrap().replace(api_token);
        }
    }

    #[cfg(feature = "real")]
    pub async fn start(
        &self,
        request: api::Request,
    ) -> Result<
        (
            api::DistributionResponse,
            Vec<String>,
            Option<Box<dyn core::future::Future<Output = Vec<String>> + Send + Unpin>>,
        ),
        String,
    > {
        let location = self.location.read().unwrap().clone();
        match location.as_ref().map(|loc| loc.as_str()) {
            Some("api") => self.distrib_api(request).await,
            Some("compose") => self.distrib_compose(request).await,
            Some(oth) => Err(format!(
                "\"{oth}\" is not a recognized distant execution location"
            )),
            None => Err("No location set".to_string()),
        }
    }

    #[cfg(feature = "mock")]
    pub async fn start(
        &self,
        request: api::Request,
    ) -> Result<
        (
            api::DistributionResponse,
            Vec<String>,
            Option<Box<dyn core::future::Future<Output = Vec<String>> + Send + Unpin>>,
        ),
        String,
    > {
        Err("Mock mode, nothing to do".to_string())
    }

    /// Reveals the API token as an `Authorization` header value.
    #[cfg(feature = "real")]
    async fn authorization(&self) -> Result<Option<Zeroizing<String>>, String> {
        let api_token = self.api_token.read().unwrap().clone();
        match api_token {
            Some(api_token) => self
                .model
                .upgrade()
                .unwrap()
                .secrets_access()
                .reveal_str(&api_token, |api_token| {
                    let mut value = String::with_capacity(7 + api_token.len());
                    value.push_str("Bearer ");
                    value.push_str(api_token);
                    Zeroizing::new(value)
                })
                .await
                .map(Some)
                .map_err(|error| format!("API token: {error}")),
            None => Ok(None),
        }
    }

    #[cfg(feature = "real")]
    async fn distrib_compose(
        &self,
        mut request: api::Request,
    ) -> Result<
        (
            api::DistributionResponse,
            Vec<String>,
            Option<Box<dyn core::future::Future<Output = Vec<String>> + Send + Unpin>>,
        ),
        String,
    > {
        request.local_exec = true;

        let mut run_api_id = None;
        let mut api_errors = Vec::new();

        let api_url = self.api_url.read().unwrap().clone();
        let api_token = self.authorization().await?;
        if let (Some(api_url), Some(api_token)) = (&api_url, &api_token) {
            match http::post(
                &format!("{api_url}/execution/run/start"),
                api_token,
                serde_json::to_string(&request).unwrap(),
            )
            .await
            {
                Ok(response) if response.status == 200 => match response.json::<api::Response>() {
                    Ok(api::Response::Ok(id)) => {
                        run_api_id = Some(id);
                        request.id = Some(id);
                    }
                    Ok(api::Response::Error(errs)) => {
                        api_errors.extend(errs);
                    }
                    Err(error) => api_errors.push(error),
                },
                Ok(response) => api_errors.push(response.error()),
                Err(error) => api_errors.push(error),
            }
        } else {
            api_errors.push("API address and token missing".into());
        }

        let response = crate::compose::compose(request).await;

        if let (Some(run_api_id), Some(api_url), Some(api_token)) =
            (run_api_id.clone(), &api_url, &api_token)
        {
            match http::post(
                &format!("{api_url}/execution/run/launched"),
                api_token,
                serde_json::to_string(&api::LocalLaunched {
                    run_id: run_api_id,
                    response: match &response {
                        Ok(_) => api::DistributionResponse::Started(None),
                        Err(errs) => api::DistributionResponse::Error(errs.clone()),
                    },
                })
                .unwrap(),
            )
            .await
            {
                Ok(response) if response.status == 200 => {}
                Ok(response) => api_errors.push(response.error()),
                Err(error) => api_errors.push(error),
            }
        }

        match response {
            Ok((access, mut child)) => {
                let finish_notification = async move {
                    let mut possible_errors = Vec::new();
                    // Once whatever signals shutdown (the container's own worker process
                    // exiting) has happened, the container itself (a `trap : TERM INT; sleep
                    // ...d & wait` loop) should tear down near-instantly - this timeout guards
                    // that teardown handshake, not the job's actual runtime, which this
                    // `.await` otherwise spans in full.
                    let status = smol::future::or(async { Some(child.status().await) }, async {
                        smol::Timer::after(Duration::from_secs(10)).await;
                        None
                    })
                    .await;
                    let (result, error) = match status {
                        Some(Ok(exit)) if exit.success() => {
                            (api::DistributionResult::Success(None), None)
                        }
                        Some(Ok(exit)) => {
                            let code = exit
                                .code()
                                .map(|code| code.to_string())
                                .unwrap_or("undefined".into());
                            (
                                api::DistributionResult::Failure(Some(vec![format!(
                                    "Compose exit code {code}"
                                )])),
                                Some(format!("Compose exited with code {code}")),
                            )
                        }
                        Some(Err(err)) => (
                            api::DistributionResult::Failure(Some(vec![err.to_string()])),
                            Some(err.to_string()),
                        ),
                        // The container never tore down within the timeout after the job
                        // finished: report the run as ended anyway (rather than leaving the
                        // API waiting forever for a call that will never come), flagging the
                        // stuck teardown as a warning rather than a job failure, since the
                        // actual work already completed successfully by this point.
                        None => {
                            let warning = "Compose exit timeout: future has timed out".to_string();
                            (
                                api::DistributionResult::Success(Some(vec![warning.clone()])),
                                Some(warning),
                            )
                        }
                    };
                    if let (Some(run_api_id), Some(api_url), Some(api_token)) =
                        (run_api_id, api_url, api_token)
                    {
                        let _ = http::post(
                            &format!("{api_url}/execution/run/ended"),
                            &api_token,
                            serde_json::to_string(&api::LocalEnd {
                                run_id: run_api_id,
                                result,
                            })
                            .unwrap(),
                        )
                        .await;
                        possible_errors.extend(error);
                    }
                    possible_errors
                };

                Ok((
                    api::DistributionResponse::Started(Some(access)),
                    api_errors,
                    Some(Box::new(Box::pin(finish_notification))),
                ))
            }
            Err(errs) => Ok((
                api::DistributionResponse::Error(errs.clone()),
                api_errors,
                None,
            )),
        }
    }

    #[cfg(feature = "real")]
    async fn distrib_api(
        &self,
        request: api::Request,
    ) -> Result<
        (
            api::DistributionResponse,
            Vec<String>,
            Option<Box<dyn core::future::Future<Output = Vec<String>> + Send + Unpin>>,
        ),
        String,
    > {
        let api_url = self.api_url.read().unwrap().clone();
        let api_token = self.authorization().await?;
        if let (Some(api_url), Some(api_token)) = (api_url, api_token) {
            let response = http::post(
                &format!("{api_url}/execution/run/start"),
                &api_token,
                serde_json::to_string(&request).unwrap(),
            )
            .await?;
            if response.status != 200 {
                return Err(response.error());
            }
            match response.json::<api::Response>()? {
                api::Response::Ok(id) => {
                    smol::Timer::after(Duration::from_secs(1)).await;
                    loop {
                        let response =
                            http::get(&format!("{api_url}/execution/run/{id}/access"), &api_token)
                                .await?;
                        match response.status {
                            202 => {
                                smol::Timer::after(Duration::from_secs(5)).await;
                            }
                            200 => {
                                return Ok((
                                    response.json::<api::DistributionResponse>()?,
                                    vec![],
                                    None,
                                ))
                            }
                            code if code < 400 => {
                                return Err(format!("API {code} response: {}", response.body))
                            }
                            _ => return Err(response.error()),
                        }
                    }
                }
                api::Response::Error(errs) => {
                    Ok((api::DistributionResponse::Error(errs), vec![], None))
                }
            }
        } else {
            Err("API address and token missing".into())
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
/// - `edition`: edition of the Mélodium engine the worker must rely on (`standard`, `desktop`, `container`, ...), can be none if nothing specific is required (any edition suits). It is not the container image, which is defined by the distribution requested in `system`; Compose runs the `standard` and `container` editions on an Alpine image, and edition only matters on container targets, workers on virtual machines satisfy any edition;
/// - `system`: what the worker must run on, either an operating system built with `|linux`, `|windows`, `|macos`, `|freebsd` and the other `work/resources/system` functions, or one exact machine image built with `|image` (a name resolved by whatever spawns the worker, never a container image); none if nothing specific is required, like for `arch`;
///
/// - `containers`: list of containers to instanciate alongside Mélodium engine as executors;
/// - `service_containers`: list of containers to instanciate alongside Mélodium engine as services;
/// - `volumes`: list of filesystem volumes that can be shared between the Mélodium engine and containers.
///
/// Every need except `tags` is a requirement: the worker is spawned satisfying it, or the request fails with a message naming the need.
/// An operating system and a machine image cannot be requested together, and when both `system` and `arch` are given the image or system must exist for that architecture.
///
/// Tags are hints: they are never a requirement and never a reason to refuse a run. Whatever spawns the worker may use them, for example by copying them to the resources it creates.
///
/// ## Secrets of containers
///
/// Containers start before the engine of the worker, so their pull secrets, and the secret variables of service containers, are given with the request:
///
/// - a secret stored on Cadence.CI (`api:<name>`) whose transmission is `reference` or `value` is given by name, never revealed, and Cadence.CI gives its value to the worker;
/// - any other secret is revealed and given by value, if its transmission is `value`, as with `std/secret::|locate<string>("env:MELODIUM_SECRET_DB_PASSWORD", "db_password", "value", false)`;
/// - other secrets, `local` ones included, are refused: `failed` is emitted, and the error names the container.
///
/// ## Where the worker is spawned
///
/// The `location` of the dispatching engine tells where the worker is spawned:
///
/// | `location` | Where the worker is spawned |
/// |---|---|
/// | `"api"` | Mélodium Services spawn the worker somewhere (container, virtual machine...) |
/// | `"compose"` | Compose: the worker is a container started on the machine running this engine, with podman or docker (tried in this order) |
///
/// Compose can only offer what its official image is. Each need is checked before anything is started:
///
/// | Need | Compose accepts | Compose refuses |
/// |---|---|---|
/// | `edition` (optional) | none, `standard`, `container` | `desktop`, `webassembly`, `mock` |
/// | `system` (optional) | none, `\|linux()`, `\|linux_distribution("alpine")` | `\|linux_distribution("ubuntu")`, `\|linux_distribution_version("alpine", "3.20")`, `\|windows()`, `\|image("nixos-ci")` |
/// | `arch` (optional) | none, the architecture of this machine | any other architecture |
/// | secrets of containers | given by value | stored on Cadence.CI (`api:`) |
/// | `tags` | anything | never refused |
///
/// A request Compose cannot satisfy starts nothing: `failed` is emitted and the errors say which need cannot be satisfied.
/// The edition is the one of the engine the worker runs, while the container image is defined by the Linux distribution of `system`.
/// Editions and systems are only requirements on container locations, a virtual machine worker satisfies any edition.
///
/// Example, asking Compose for a `standard` engine on Alpine:
///
/// ```melodium
/// use std/engine/util::startup
/// use std/engine/log::logErrors
/// use std/ops/option::|wrap
/// use work/distant::DistantEngine
/// use work/distant::distant
/// use work/resources/system::System
/// use work/resources/system::|linux_distribution
///
/// treatment main()
///   model engine: DistantEngine(location="compose")
/// {
///     startup()
///     request: distant[distant_engine=engine](
///         max_duration=60, memory=500, cpu=500, storage=500,
///         edition="standard",
///         system=|wrap<System>(|linux_distribution("alpine")),
///         arch=_, volumes=[], containers=[], service_containers=[], tags=["example"]
///     )
///     startup.trigger -> request.trigger
///     logRequestErrors: logErrors(label="distant")
///     request.errors -> logRequestErrors.messages
/// }
/// ```
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
    secrets_access
    default system none
)]
pub async fn distant(
    max_duration: u32,
    memory: u32,
    cpu: u32,
    storage: u32,
    edition: Option<string>,
    arch: Option<Arch>,
    system: Option<System>,
    volumes: Vec<Volume>,
    containers: Vec<Container>,
    service_containers: Vec<ServiceContainer>,
    tags: Vec<string>,
) {
    let model = DistantEngineModel::into(distant_engine);
    let distant = model.inner();

    // Secrets of containers are given with the request, by name or by value.
    let mut request_containers = Vec::with_capacity(containers.len());
    for Container(mut container, pull_secret) in containers.into_iter().map(Arc::unwrap_or_clone) {
        if let Some(pull_secret) = pull_secret {
            match container_secret(&secrets_access, &pull_secret).await {
                Ok(ContainerSecret::Name(name)) => container.pull_secret_name = Some(name),
                Ok(ContainerSecret::Value(value)) => container.pull_secret = value.to_string(),
                Err(error) => {
                    let _ = failed.send_one_as(()).await;
                    let _ = errors
                        .send_one_as(format!(
                            "pull secret of container '{}': {error}",
                            container.name
                        ))
                        .await;
                    return;
                }
            }
        }
        request_containers.push(container);
    }
    let mut request_service_containers = Vec::with_capacity(service_containers.len());
    for ServiceContainer(mut container, pull_secret, secret_env) in
        service_containers.into_iter().map(Arc::unwrap_or_clone)
    {
        let secrets = pull_secret
            .map(|secret| ("pull secret".to_string(), None, Some(secret)))
            .into_iter()
            .chain(secret_env.into_iter().map(|(variable, secret)| {
                (format!("variable '{variable}'"), Some(variable), secret)
            }));
        for (what, variable, secret) in secrets {
            let given = match secret {
                Some(secret) => container_secret(&secrets_access, &secret).await,
                None => Err(SecretError::MismatchingValue),
            };
            match (given, variable) {
                (Ok(ContainerSecret::Name(name)), Some(variable)) => {
                    container.env.remove(&variable);
                    container.secret_env.insert(variable, name);
                }
                (Ok(ContainerSecret::Value(value)), Some(variable)) => {
                    container.env.insert(variable, value.to_string());
                }
                (Ok(ContainerSecret::Name(name)), None) => container.pull_secret_name = Some(name),
                (Ok(ContainerSecret::Value(value)), None) => {
                    container.pull_secret = value.to_string()
                }
                (Err(error), _) => {
                    let _ = failed.send_one_as(()).await;
                    let _ = errors
                        .send_one_as(format!(
                            "{what} of service container '{}': {error}",
                            container.name
                        ))
                        .await;
                    return;
                }
            }
        }
        request_service_containers.push(container);
    }

    let key = Uuid::new_v4();
    let start = api::Request {
        edition,
        max_duration: Some(max_duration),
        memory: Some(memory),
        cpu: Some(cpu),
        mode: api::ModeRequest::DistributionSecretKey { key: key.clone() },
        config: None,
        id: None,
        organization_id: None,
        version: env!("CARGO_PKG_VERSION").to_string(),
        storage: Some(storage),
        arch: arch.map(|arch| arch.0),
        system: api::SystemRequest(system.map(|system| system.0.clone())),
        volumes: volumes.into_iter().map(|vol| vol.0.clone()).collect(),
        containers: request_containers,
        service_containers: request_service_containers,
        group_id: Some(melodium_engine::execution_group_id().clone()),
        parent_id: Some(melodium_engine::execution_run_id().clone()),
        tags: tags,
        local_exec: false,
    };

    if let Ok(_) = trigger.recv_one().await {
        match distant.start(start).await {
            Ok((distrib, api_errors, future)) => {
                let _ = errors.send_many_as(api_errors).await;
                match distrib {
                    api::DistributionResponse::Started(Some(access_info)) => {
                        let _ = access
                            .send_one(Value::Data(Arc::new(Access::with_keys(
                                access_info.addresses,
                                access_info.port,
                                access_info.key,
                                key,
                                access_info.disable_tls,
                                access_info.allow_plain_tcp,
                            ))))
                            .await;
                        let _ = access.close().await;
                        let _ = failed.close().await;

                        if let Some(future_errors) = future {
                            let some_errors = future_errors.await;
                            if !some_errors.is_empty() {
                                let _ = errors.send_many_as(some_errors).await;
                            }
                        }

                        let _ = errors.close().await;
                    }
                    api::DistributionResponse::Started(None) => {}
                    api::DistributionResponse::Error(errs) => {
                        let _ = failed.send_one_as(()).await;
                        let _ = errors.send_many_as(errs).await;
                    }
                }
            }
            Err(err) => {
                let _ = failed.send_one_as(()).await;
                let _ = errors.send_many_as(vec![err]).await;
            }
        }
    }
}

/// What a worker request gives for a secret of one of its containers.
enum ContainerSecret {
    /// Name of the secret stored on Cadence.CI, whose value Cadence.CI gives to the worker.
    Name(String),
    /// Revealed value.
    Value(Zeroizing<String>),
}

/// Gives what the worker request gives for `secret`, a pull secret or a secret variable of
/// a container, as `distant` documents it.
async fn container_secret(
    access: &SecretsAccess,
    secret: &ExecutiveSecret,
) -> Result<ContainerSecret, SecretError> {
    let stored = secret
        .locator()
        .and_then(|locator| locator.strip_prefix(&format!("{}:", crate::secret::SCHEME)));
    match (secret.policy().transmission, stored) {
        (SecretTransmission::Local, _) => Err(SecretError::Denied(
            "its policy keeps it on this engine".to_string(),
        )),
        (_, Some(name)) => {
            // Sent by reference, so recorded as such and never revealed.
            let by_reference = secret.narrow(&SecretPolicy {
                transmission: SecretTransmission::Reference,
                ..SecretPolicy::unrestricted()
            });
            match access.transmit(&by_reference, true).await? {
                SecretTransfer::Reference(_) => Ok(ContainerSecret::Name(name.to_string())),
                // A secret sent by reference gives its locator, not its value.
                SecretTransfer::Value(mut value) => {
                    wipe_value(&mut value);
                    Err(SecretError::Denied(
                        "it cannot be given by reference".to_string(),
                    ))
                }
            }
        }
        (SecretTransmission::Value, None) => {
            access
                .reveal_str(secret, |value| {
                    ContainerSecret::Value(Zeroizing::new(value.to_string()))
                })
                .await
        }
        (SecretTransmission::Reference, None) => Err(SecretError::Denied(format!(
            "containers only get secrets by reference when stored on Cadence.CI, located '{}:<name>'",
            crate::secret::SCHEME
        ))),
    }
}

/// Return the default Mélodium Services API URL.
#[mel_function]
pub fn default_api_url() -> string {
    crate::API_URL.to_string()
}
