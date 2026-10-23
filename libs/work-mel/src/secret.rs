//! Secrets stored on Cadence.CI, given to the programs it runs through `api:<name>` locators,
//! and stored by them, and identity tokens of runs, given through `api-id:<audience>` locators.

use async_trait::async_trait;
use core::fmt::{Debug, Formatter};
use melodium_core::common::descriptor::DataType;
use melodium_core::common::executive::{
    count_wiped, register_wipe, PackedArray, Secret as ExecutiveSecret, SecretSource,
    SecretsAccess, Wipe,
};
use melodium_core::*;
use melodium_macro::mel_treatment;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError, Weak};
use std::time::{Duration, SystemTime};
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

/// Scheme of the locators of secrets stored on Cadence.CI.
pub const SCHEME: &str = "api";
/// Scheme of the locators of the identity tokens of runs.
pub const IDENTITY_SCHEME: &str = "api-id";

/// Time left to an identity token below which a new one is requested.
const IDENTITY_TOKEN_MARGIN: Duration = Duration::from_secs(30);

/// Longest secret name the API takes.
const MAX_NAME_LENGTH: usize = 200;
/// Longest reason of a refusal given in errors, in characters.
#[cfg(feature = "real")]
const MAX_REASON_LENGTH: usize = 300;

/// Sources alive in this process, whose cached values become stale when a secret is stored.
static SOURCES: Mutex<Vec<Weak<ApiSource>>> = Mutex::new(Vec::new());

/// How secrets are read and stored through the API.
struct Api {
    url: String,
    token: Zeroizing<String>,
    run_id: Uuid,
}

impl Api {
    /// Reaches the API as configured on Cadence.CI workers, or tells what is missing.
    fn from_environment() -> Result<Self, String> {
        let url = std::env::var("MELODIUM_API_URL").ok();
        let token = std::env::var("MELODIUM_API_TOKEN").ok().map(Zeroizing::new);
        let run_id = std::env::var("MELODIUM_RUN_ID").ok();
        match (url, token, run_id) {
            (Some(url), Some(token), Some(run_id)) => Uuid::parse_str(&run_id)
                .map(|run_id| Api { url, token, run_id })
                .map_err(|_| "MELODIUM_RUN_ID is not a run identifier".to_string()),
            (url, token, run_id) => {
                let missing = [
                    ("MELODIUM_API_URL", url.is_none()),
                    ("MELODIUM_API_TOKEN", token.is_none()),
                    ("MELODIUM_RUN_ID", run_id.is_none()),
                ]
                .into_iter()
                .filter_map(|(variable, missing)| missing.then_some(variable))
                .collect::<Vec<_>>();
                Err(format!(
                    "{} {} not set",
                    missing.join(", "),
                    if missing.len() > 1 { "are" } else { "is" }
                ))
            }
        }
    }

    /// URL of the secret `name` of the run.
    #[cfg(feature = "real")]
    fn secret_url(&self, name: &str) -> String {
        format!(
            "{}/execution/run/{}/secret/{name}",
            self.url.trim_end_matches('/'),
            self.run_id
        )
    }
}

/// `api:<name>` source, giving the secret `name` stored on Cadence.CI to the run reading it.
///
/// The API is reached as configured on Cadence.CI workers: at `MELODIUM_API_URL`, with the
/// token `MELODIUM_API_TOKEN` of the run `MELODIUM_RUN_ID`. The token configures Mélodium
/// itself, programs never get it.
///
/// Each value is read once, then kept for the life of the engine: it is wiped when the
/// engine ends, and before the process exits. `Secret<string>` values must be valid UTF-8,
/// `Secret<Vec<byte>>` ones are given as stored.
///
/// Its secrets are never plainly revealed: a program only gets them as secrets, revealed by
/// the elements using them.
pub struct ApiSource {
    /// How to reach the API, or what is missing to.
    api: Result<Api, String>,
    cache: Mutex<HashMap<String, Zeroizing<Vec<u8>>>>,
    /// Identity tokens, by audience, with the instant they expire.
    tokens: Mutex<HashMap<String, (Zeroizing<String>, SystemTime)>>,
}

impl ApiSource {
    /// Source reading the secrets of the run `run_id` from the API at `url`, with `token`.
    pub fn new(url: String, token: String, run_id: Uuid) -> Arc<Self> {
        Self::with(Ok(Api {
            url,
            token: Zeroizing::new(token),
            run_id,
        }))
    }

    /// Source configured from the environment, as on Cadence.CI workers.
    ///
    /// Elsewhere, it refuses every secret, telling what is missing.
    pub fn from_environment() -> Arc<Self> {
        Self::with(Api::from_environment())
    }

    fn with(api: Result<Api, String>) -> Arc<Self> {
        let source = Arc::new(Self {
            api,
            cache: Mutex::new(HashMap::new()),
            tokens: Mutex::new(HashMap::new()),
        });
        register_wipe(Arc::downgrade(&source) as Weak<dyn Wipe>);
        let mut sources = SOURCES.lock().unwrap_or_else(PoisonError::into_inner);
        sources.retain(|source| source.strong_count() > 0);
        sources.push(Arc::downgrade(&source));
        source
    }

    /// Forgets the value of `name` kept by every source, once a new one is stored.
    fn forget(name: &str) {
        for source in SOURCES
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .filter_map(Weak::upgrade)
        {
            // The value overwrites itself when dropped.
            source
                .cache
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(name);
        }
    }
}

#[async_trait]
impl SecretSource for ApiSource {
    fn plain_reveal(&self) -> bool {
        false
    }

    async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String> {
        let api = self.api.as_ref().map_err(|problem| {
            format!(
                "'{SCHEME}:' secrets are only given to programs run by Cadence.CI, and {problem}"
            )
        })?;
        check_name(path)?;
        let as_bytes = match datatype {
            DataType::String => false,
            DataType::Vec(inner) if matches!(**inner, DataType::Byte) => true,
            other => return Err(format!("secret '{path}' cannot be given as {other}")),
        };

        let cached = self
            .cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(path)
            .map(|content| Zeroizing::new(content.to_vec()));
        let mut content = match cached {
            Some(content) => content,
            None => {
                let content = fetch(api, path).await?;
                self.cache
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(path.to_string(), Zeroizing::new(content.to_vec()));
                content
            }
        };

        let content = std::mem::take(&mut *content);
        if as_bytes {
            Ok(Value::Packed(PackedArray::Byte(Arc::new(content))))
        } else {
            String::from_utf8(content)
                .map(Value::String)
                .map_err(|err| {
                    err.into_bytes().zeroize();
                    format!("secret '{path}' is not valid UTF-8, it can only be given as Vec<byte>")
                })
        }
    }

    fn shutdown(&self) {
        self.wipe();
    }
}

impl Wipe for ApiSource {
    fn wipe(&self) -> bool {
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        let mut tokens = self.tokens.lock().unwrap_or_else(PoisonError::into_inner);
        let wiped = !cache.is_empty() || !tokens.is_empty();
        // Values overwrite themselves when dropped.
        cache.clear();
        tokens.clear();
        if wiped {
            count_wiped();
        }
        wiped
    }
}

/// `api-id:<audience>` source, giving the OIDC identity token of the run for `audience`,
/// such as `api-id:vault`, configured as `api:`.
///
/// Cadence.CI signs these tokens for each run, valid a few minutes, so that clouds and Vault
/// trusting Cadence.CI let programs in without long-lived keys. A token is kept until shortly
/// before it expires, then a new one is requested; tokens are wiped when the engine ends, and
/// before the process exits.
///
/// Its secrets are never plainly revealed.
#[derive(Debug)]
pub struct ApiIdentitySource(Arc<ApiSource>);

impl ApiIdentitySource {
    /// Source of identity tokens, reaching the API as `source` does.
    pub fn new(source: &Arc<ApiSource>) -> Arc<Self> {
        Arc::new(Self(Arc::clone(source)))
    }
}

#[async_trait]
impl SecretSource for ApiIdentitySource {
    fn plain_reveal(&self) -> bool {
        false
    }

    async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String> {
        let source = &self.0;
        let api = source.api.as_ref().map_err(|problem| {
            format!(
                "'{IDENTITY_SCHEME}:' tokens are only given to programs run by Cadence.CI, and {problem}"
            )
        })?;
        if datatype != &DataType::String {
            return Err(format!(
                "identity tokens are given as string, not as {datatype}"
            ));
        }
        if path.is_empty() {
            return Err(format!(
                "identity tokens need an audience, as in '{IDENTITY_SCHEME}:vault'"
            ));
        }

        let renew_at = SystemTime::now() + IDENTITY_TOKEN_MARGIN;
        {
            let mut tokens = source.tokens.lock().unwrap_or_else(PoisonError::into_inner);
            // The token overwrites itself when dropped.
            tokens.retain(|_, (_, expires)| *expires > renew_at);
            if let Some((token, _)) = tokens.get(path) {
                return Ok(Value::String(token.to_string()));
            }
        }

        let token = fetch_identity_token(api, path).await?;
        // A token whose expiry cannot be read is not kept.
        if let Some(expires) = expiry(&token).filter(|expires| *expires > renew_at) {
            source
                .tokens
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(path.to_string(), (token.clone(), expires));
        }
        Ok(Value::String(token.to_string()))
    }

    fn shutdown(&self) {
        self.0.wipe();
    }
}

/// Gives the instant `token`, a JSON Web Token, expires, from its `exp` claim.
#[cfg(feature = "real")]
fn expiry(token: &str) -> Option<SystemTime> {
    use base64::Engine;

    let payload = token.split('.').nth(1)?;
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload.trim_end_matches('='))
        .ok()?;
    let expires = serde_json::from_slice::<serde_json::Value>(&payload)
        .ok()?
        .get("exp")?
        .as_u64()?;
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(expires))
}

#[cfg(feature = "mock")]
fn expiry(_token: &str) -> Option<SystemTime> {
    None
}

// The token and the values are never shown.
impl Debug for ApiSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ApiSource").finish_non_exhaustive()
    }
}

/// Checks that `name` is a secret name of Cadence.CI: parts of letters, digits, `_`, `.` and
/// `-`, separated by `/`. Neither `.` nor `..` are parts, so that no name reaches another
/// endpoint of the API.
fn check_name(name: &str) -> Result<(), String> {
    let valid = !name.is_empty()
        && name.len() <= MAX_NAME_LENGTH
        && name.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
        });
    if valid {
        Ok(())
    } else {
        Err(format!(
            "'{name}' is not a secret name of Cadence.CI, made of letters, digits, '_', '.' and '-', in parts separated by '/'"
        ))
    }
}

/// Stores a secret on Cadence.CI, for later runs to read it through `api:<name>`.
///
/// The secret received through `secret` is revealed only to be sent to Cadence.CI, as the
/// secret `name` of the organization of the run, as a new version when it already exists.
/// Names are made of letters, digits, `_`, `.` and `-`, in parts separated by `/`.
///
/// Only programs run by Cadence.CI store secrets, when the rules of their organization allow
/// it: a rule letting runs write must match `name`, and public runs never store any.
/// Store credentials a program produces and later runs need, such as a generated database
/// password or the kubeconfig of a cluster it created, rather than putting them in artifacts.
///
/// `completed` is emitted once the secret is stored. Otherwise `failed` is emitted, and `error`
/// gives the reason, such as the rule refusing it.
#[mel_treatment(
    input secret Block<Secret<string>>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    secrets_access
)]
pub async fn store(name: string) {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        match store_secret(&secrets_access, &name, &received, false).await {
            Ok(()) => {
                let _ = completed.send_one(().into()).await;
            }
            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(err.into()).await;
            }
        }
    }
}

/// Stores a binary secret on Cadence.CI, for later runs to read it through `api:<name>`.
///
/// Same as `store`, for a `Secret<Vec<byte>>`, such as a key file.
#[mel_treatment(
    input secret Block<Secret<Vec<byte>>>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    secrets_access
)]
pub async fn store_bytes(name: string) {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        match store_secret(&secrets_access, &name, &received, true).await {
            Ok(()) => {
                let _ = completed.send_one(().into()).await;
            }
            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(err.into()).await;
            }
        }
    }
}

/// Stores `secret` as `name`, revealing it only to send it, giving why it could not be.
async fn store_secret(
    access: &SecretsAccess,
    name: &str,
    secret: &ExecutiveSecret,
    bytes: bool,
) -> Result<(), String> {
    let api = Api::from_environment().map_err(|problem| {
        format!("secrets are only stored on Cadence.CI by the programs it runs, and {problem}")
    })?;
    check_name(name)?;
    let value = if bytes {
        access
            .reveal_bytes(secret, |value| Zeroizing::new(value.to_vec()))
            .await
    } else {
        access
            .reveal_str(secret, |value| Zeroizing::new(value.as_bytes().to_vec()))
            .await
    }
    .map_err(|err| format!("secret {secret}: {err}"))?;

    put(&api, name, value).await?;
    ApiSource::forget(name);
    Ok(())
}

/// Describes a refusal of the API, from its status and body.
#[cfg(feature = "real")]
fn refusal(status: reqwest::StatusCode, body: &[u8]) -> String {
    // The API gives the reason of a refusal, never a value. Proxies in between can answer
    // whole pages instead.
    let mut reason = String::from_utf8_lossy(body)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if let Some((cut, _)) = reason.char_indices().nth(MAX_REASON_LENGTH) {
        reason.truncate(cut);
        reason.push('…');
    }
    if reason.is_empty() {
        format!("Cadence.CI answered {status}")
    } else {
        format!("Cadence.CI answered {status}: {reason}")
    }
}

/// Sends `value` to the API, as the new value of the secret `name`.
#[cfg(feature = "real")]
async fn put(api: &Api, name: &str, value: Zeroizing<Vec<u8>>) -> Result<(), String> {
    use async_compat::Compat;
    use reqwest::header::CONTENT_TYPE;

    let client = crate::http::CLIENT.as_ref().map_err(Clone::clone)?;
    // Held apart, to be overwritten once sent.
    let body = bytes::Bytes::from(value.to_vec());
    drop(value);

    let mut result = Err(String::new());
    // A conflict tells someone else created the secret meanwhile: storing again makes a new
    // version of it.
    for _ in 0..2 {
        let request = client
            .put(api.secret_url(name))
            .bearer_auth(api.token.as_str())
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(body.clone());
        // The HTTP client runs on Tokio.
        result = Compat::new(async {
            let response = request.send().await?;
            let status = response.status();
            response.bytes().await.map(|answer| (status, answer))
        })
        .await
        .map_err(|err| format!("Cadence.CI cannot be reached: {err}"))
        .and_then(|(status, answer)| {
            if status.is_success() {
                Ok(())
            } else {
                Err(refusal(status, &answer))
            }
        });
        if !matches!(&result, Err(error) if error.starts_with("Cadence.CI answered 409")) {
            break;
        }
    }

    if let Ok(mut body) = body.try_into_mut() {
        body.as_mut().zeroize();
    }
    result
}

#[cfg(feature = "mock")]
async fn put(_api: &Api, name: &str, _value: Zeroizing<Vec<u8>>) -> Result<(), String> {
    Err(format!(
        "secret '{name}' cannot be stored on Cadence.CI in this build"
    ))
}

/// Requests an identity token of the run for `audience` from the API.
#[cfg(feature = "real")]
async fn fetch_identity_token(api: &Api, audience: &str) -> Result<Zeroizing<String>, String> {
    use async_compat::Compat;

    let request = crate::http::CLIENT
        .as_ref()
        .map_err(Clone::clone)?
        .get(format!(
            "{}/execution/run/{}/id-token",
            api.url.trim_end_matches('/'),
            api.run_id
        ))
        .query(&[("audience", audience)])
        .bearer_auth(api.token.as_str());

    // The HTTP client runs on Tokio.
    let (status, body) = Compat::new(async {
        let response = request.send().await?;
        let status = response.status();
        response.bytes().await.map(|body| (status, body))
    })
    .await
    .map_err(|err| format!("Cadence.CI cannot be reached: {err}"))?;
    let content = Zeroizing::new(body.to_vec());
    if let Ok(mut body) = body.try_into_mut() {
        body.as_mut().zeroize();
    }

    if !status.is_success() {
        return Err(refusal(status, &content));
    }
    match std::str::from_utf8(&content).map(str::trim) {
        Ok(token) if !token.is_empty() => Ok(Zeroizing::new(token.to_string())),
        _ => Err("Cadence.CI gave no identity token".to_string()),
    }
}

#[cfg(feature = "mock")]
async fn fetch_identity_token(_api: &Api, audience: &str) -> Result<Zeroizing<String>, String> {
    Err(format!(
        "identity tokens for '{audience}' cannot be requested from Cadence.CI in this build"
    ))
}

/// Reads the value of the secret `name` from the API.
#[cfg(feature = "real")]
async fn fetch(api: &Api, name: &str) -> Result<Zeroizing<Vec<u8>>, String> {
    use async_compat::Compat;

    let request = crate::http::CLIENT
        .as_ref()
        .map_err(Clone::clone)?
        .get(api.secret_url(name))
        .bearer_auth(api.token.as_str());

    // The HTTP client runs on Tokio.
    let (status, body) = Compat::new(async {
        let response = request.send().await?;
        let status = response.status();
        response.bytes().await.map(|body| (status, body))
    })
    .await
    .map_err(|err| format!("Cadence.CI cannot be reached: {err}"))?;
    let content = Zeroizing::new(body.to_vec());
    if let Ok(mut body) = body.try_into_mut() {
        body.as_mut().zeroize();
    }

    if status.is_success() {
        Ok(content)
    } else {
        Err(refusal(status, &content))
    }
}

#[cfg(feature = "mock")]
async fn fetch(_api: &Api, name: &str) -> Result<Zeroizing<Vec<u8>>, String> {
    Err(format!(
        "secret '{name}' cannot be read from Cadence.CI in this build"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_secrets_are_forgotten_by_every_source() {
        let run_id = Uuid::new_v4();
        let sources = [
            ApiSource::new("http://127.0.0.1:1".to_string(), String::new(), run_id),
            ApiSource::new("http://127.0.0.1:1".to_string(), String::new(), run_id),
        ];
        for source in &sources {
            let mut cache = source.cache.lock().unwrap();
            cache.insert("stored".to_string(), Zeroizing::new(b"old".to_vec()));
            cache.insert("other".to_string(), Zeroizing::new(b"kept".to_vec()));
        }

        ApiSource::forget("stored");
        for source in &sources {
            let cache = source.cache.lock().unwrap();
            assert!(!cache.contains_key("stored"));
            assert!(cache.contains_key("other"));
        }
    }
}
