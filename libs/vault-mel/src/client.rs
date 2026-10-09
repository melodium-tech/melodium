use crate::VaultModel;
use async_std::sync::Mutex;
use generic_async_http_client::Request;
use melodium_core::common::descriptor::DataType;
use melodium_core::common::executive::{PackedArray, Secret, Value};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde_json::{Map, Value as Json};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

const USER_AGENT: &str = concat!("vault-mel/", env!("CARGO_PKG_VERSION"));

/// Error of a request to the server, never containing secret values.
enum RequestError {
    Status(u16, String),
    Other(String),
}

impl RequestError {
    fn message(self) -> String {
        match self {
            RequestError::Status(status, errors) => {
                format!("vault answered with status {status}{errors}")
            }
            RequestError::Other(message) => message,
        }
    }
}

/// Client token, with the instant it should be renewed, if it expires.
type Token = (String, Option<Instant>);

/// Data of a secret, with the instant it was read.
type Cached = (Instant, Arc<Map<String, Json>>);

pub struct Client {
    model: Weak<VaultModel>,
    token: Mutex<Option<Token>>,
    cache: Mutex<HashMap<String, Cached>>,
}

// Token and cached secrets are never shown.
impl core::fmt::Debug for Client {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Client").finish_non_exhaustive()
    }
}

impl Client {
    pub fn new(model: Weak<VaultModel>) -> Self {
        Self {
            model,
            token: Mutex::new(None),
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn model(&self) -> Result<Arc<VaultModel>, String> {
        self.model
            .upgrade()
            .ok_or_else(|| "vault model has ended".to_string())
    }

    /// Gives the value at `path` (`<path>#<field>`) as `datatype`.
    pub async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String> {
        let (path, field) = match path.rsplit_once('#') {
            Some((path, field)) => (path, Some(field)),
            None => (path, None),
        };
        let data = self.read(path).await?;
        let value = match field {
            Some(field) => data
                .get(field)
                .ok_or_else(|| format!("vault secret '{path}' has no field '{field}'"))?,
            None if data.len() == 1 => data.values().next().unwrap(),
            None => {
                let message = format!("vault secret '{path}' has several fields");
                return Err(format!("{message}, one must be given as '{path}#<field>'"));
            }
        };
        let text = match value {
            Json::String(text) => text.clone(),
            other => other.to_string(),
        };
        match datatype {
            DataType::String => Ok(Value::String(text)),
            DataType::Vec(inner) if matches!(**inner, DataType::Byte) => Ok(Value::Packed(
                PackedArray::Byte(Arc::new(text.into_bytes())),
            )),
            other => Err(format!("vault secret '{path}' cannot be given as {other}")),
        }
    }

    /// Gives the data of the secret at `path`, from the cache if still valid.
    pub async fn read(&self, path: &str) -> Result<Arc<Map<String, Json>>, String> {
        let ttl = Duration::from_secs(self.model()?.get_cache_ttl());
        if !ttl.is_zero() {
            let mut cache = self.cache.lock().await;
            cache.retain(|_, (read_at, _)| read_at.elapsed() < ttl);
            if let Some((_, data)) = cache.get(path) {
                return Ok(Arc::clone(data));
            }
        }

        let data = self.fetch(path).await?;
        if !ttl.is_zero() {
            self.cache
                .lock()
                .await
                .insert(path.to_string(), (Instant::now(), Arc::clone(&data)));
        }
        Ok(data)
    }

    /// Forgets the client token and the cached secrets.
    pub async fn forget(&self) {
        *self.token.lock().await = None;
        self.cache.lock().await.clear();
    }

    async fn fetch(&self, path: &str) -> Result<Arc<Map<String, Json>>, String> {
        let token = self.client_token(false).await?;
        let response = match self.get(path, &token).await {
            // An expired or revoked token gets one new login.
            Err(RequestError::Status(403, _)) if self.model()?.get_auth() != "token" => {
                let token = self.client_token(true).await?;
                self.get(path, &token).await
            }
            other => other,
        }
        .map_err(RequestError::message)?;

        let data = response
            .get("data")
            .and_then(Json::as_object)
            .ok_or_else(|| format!("vault secret '{path}' has no data"))?;
        // Key/value version 2 nests the secret data with its metadata.
        let data = match (data.get("data"), data.get("metadata")) {
            (Some(Json::Object(data)), Some(_)) => data,
            _ => data,
        };
        Ok(Arc::new(data.clone()))
    }

    async fn get(&self, path: &str, token: &str) -> Result<Json, RequestError> {
        let model = self.model().map_err(RequestError::Other)?;
        let request = Request::get(&format!(
            "{}/v1/{}",
            model.get_address().trim_end_matches('/'),
            path.trim_start_matches('/')
        ))
        .add_header("X-Vault-Token", token)
        .map_err(|err| RequestError::Other(err.to_string()))?;
        self.send(request).await
    }

    async fn send(&self, request: Request) -> Result<Json, RequestError> {
        let model = self.model().map_err(RequestError::Other)?;
        let mut request = request
            .add_header("User-Agent", USER_AGENT)
            .map_err(|err| RequestError::Other(err.to_string()))?;
        let namespace = model.get_namespace();
        if !namespace.is_empty() {
            request = request
                .add_header("X-Vault-Namespace", namespace.as_str())
                .map_err(|err| RequestError::Other(err.to_string()))?;
        }

        // Client and server error statuses come as errors, with their response.
        let (status, mut response) = match request.exec().await {
            Ok(response) => (response.status_code(), response),
            Err(generic_async_http_client::Error::HTTPClientErr(status, response))
            | Err(generic_async_http_client::Error::HTTPServerErr(status, response)) => {
                (status, response)
            }
            Err(err) => return Err(RequestError::Other(format!("vault request failed: {err}"))),
        };
        let body = response
            .content()
            .await
            .map_err(|err| RequestError::Other(format!("vault response failed: {err}")))?;
        // Parsing errors are not reported as is, they can quote the content.
        let json = serde_json::from_slice::<Json>(&body).ok();
        if (200..300).contains(&status) {
            json.ok_or_else(|| RequestError::Other("vault response is not valid JSON".to_string()))
        } else {
            let errors = json
                .as_ref()
                .and_then(|json| json.get("errors"))
                .and_then(Json::as_array)
                .map(|errors| {
                    errors
                        .iter()
                        .filter_map(Json::as_str)
                        .map(|error| error.split_whitespace().collect::<Vec<_>>().join(" "))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .filter(|errors| !errors.is_empty())
                .map(|errors| format!(": {errors}"))
                .unwrap_or_default();
            Err(RequestError::Status(status, errors))
        }
    }

    /// Gives a client token, logging in if needed or if `renew`.
    async fn client_token(&self, renew: bool) -> Result<String, String> {
        let mut token = self.token.lock().await;
        if let Some((value, renew_at)) = &*token {
            if !renew && renew_at.map(|at| Instant::now() < at).unwrap_or(true) {
                return Ok(value.clone());
            }
        }
        let new = self.login().await?;
        let value = new.0.clone();
        *token = Some(new);
        Ok(value)
    }

    async fn reveal(&self, secret: Secret) -> Result<String, String> {
        let model = self.model()?;
        // Reading it would need the login it is used for.
        let own_locators = format!("{}:", model.get_source());
        if std::iter::once(secret.clone())
            .chain(secret.derived_from())
            .any(|secret| {
                secret
                    .locator()
                    .map(|locator| locator.starts_with(&own_locators))
                    .unwrap_or(false)
            })
        {
            return Err(format!(
                "secret {secret} cannot come from the vault it authenticates to"
            ));
        }
        model
            .secrets_access()
            .reveal_str(&secret, |value| value.to_string())
            .await
            .map_err(|err| format!("secret {secret}: {err}"))
    }

    async fn login(&self) -> Result<Token, String> {
        let model = self.model()?;
        let auth = model.get_auth();
        let body = match auth.as_str() {
            "token" => return Ok((self.reveal(model.get_token()).await?, None)),
            "approle" => serde_json::json!({
                "role_id": self.reveal(model.get_role_id()).await?,
                "secret_id": self.reveal(model.get_secret_id()).await?,
            }),
            "kubernetes" => serde_json::json!({
                "role": model.get_role(),
                "jwt": self.reveal(model.get_kubernetes_token()).await?,
            }),
            "jwt" => serde_json::json!({
                "role": model.get_role(),
                "jwt": self.reveal(model.get_jwt()).await?,
            }),
            "github" => serde_json::json!({
                "role": model.get_role(),
                "jwt": self.github_token().await?,
            }),
            other => {
                let expected = "token, approle, kubernetes, jwt or github";
                return Err(format!(
                    "unknown vault authentication '{other}', expected {expected}"
                ));
            }
        };
        let mount = match (model.get_auth_mount(), auth.as_str()) {
            (mount, _) if !mount.is_empty() => mount,
            (_, "github") => "jwt".to_string(),
            (_, auth) => auth.to_string(),
        };

        let request = Request::post(&format!(
            "{}/v1/auth/{}/login",
            model.get_address().trim_end_matches('/'),
            mount.trim_matches('/')
        ))
        .json(&body)
        .map_err(|err| err.to_string())?;
        let response = self
            .send(request)
            .await
            .map_err(|err| format!("vault login failed, {}", err.message()))?;

        let auth = response
            .get("auth")
            .ok_or_else(|| "vault login gave no token".to_string())?;
        let token = auth
            .get("client_token")
            .and_then(Json::as_str)
            .ok_or_else(|| "vault login gave no token".to_string())?
            .to_string();
        model
            .secrets_access()
            .add_masked_value("vault token", &Value::String(token.clone()));
        // Renewed a bit before it expires, a lease of 0 never expires.
        let renew_at = auth
            .get("lease_duration")
            .and_then(Json::as_u64)
            .filter(|lease| *lease > 0)
            .map(|lease| Instant::now() + Duration::from_secs(lease) * 9 / 10);
        Ok((token, renew_at))
    }

    /// Requests an OIDC token from GitHub Actions.
    async fn github_token(&self) -> Result<String, String> {
        let model = self.model()?;
        let url = std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL").map_err(|_| {
            "ACTIONS_ID_TOKEN_REQUEST_URL is not set, the job needs the `id-token: write` permission"
                .to_string()
        })?;
        let audience = model.get_github_audience();
        let url = if audience.is_empty() {
            url
        } else {
            format!(
                "{url}&audience={}",
                utf8_percent_encode(&audience, NON_ALPHANUMERIC)
            )
        };
        let request_token = self.reveal(model.get_github_token()).await?;
        let request = Request::get(&url)
            .add_header("Authorization", format!("bearer {request_token}").as_str())
            .map_err(|err| err.to_string())?;
        let response = self
            .send(request)
            .await
            .map_err(|err| format!("GitHub OIDC token request failed, {}", err.message()))?;
        let token = response
            .get("value")
            .and_then(Json::as_str)
            .ok_or_else(|| "GitHub OIDC token request gave no token".to_string())?
            .to_string();
        model
            .secrets_access()
            .add_masked_value("GitHub OIDC token", &Value::String(token.clone()));
        Ok(token)
    }
}
