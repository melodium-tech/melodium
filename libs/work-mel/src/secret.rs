//! Secrets stored on Cadence.CI, given to the programs it runs through `api:<name>` locators.

use async_trait::async_trait;
use core::fmt::{Debug, Formatter};
use melodium_core::common::descriptor::DataType;
use melodium_core::common::executive::{
    count_wiped, register_wipe, PackedArray, SecretSource, Value, Wipe,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError, Weak};
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

/// Scheme of the locators of secrets stored on Cadence.CI.
pub const SCHEME: &str = "api";

/// Longest secret name the API takes.
const MAX_NAME_LENGTH: usize = 200;
/// Longest reason of a refusal given in errors, in characters.
#[cfg(feature = "real")]
const MAX_REASON_LENGTH: usize = 300;

/// How the source reaches the API.
struct Api {
    url: String,
    token: Zeroizing<String>,
    run_id: Uuid,
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
    api: Result<Api, String>,
    cache: Mutex<HashMap<String, Zeroizing<Vec<u8>>>>,
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
        let url = std::env::var("MELODIUM_API_URL").ok();
        let token = std::env::var("MELODIUM_API_TOKEN").ok().map(Zeroizing::new);
        let run_id = std::env::var("MELODIUM_RUN_ID").ok();
        let api = match (url, token, run_id) {
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
                    "'{SCHEME}:' secrets are only given to programs run by Cadence.CI, and {} {} not set",
                    missing.join(", "),
                    if missing.len() > 1 { "are" } else { "is" }
                ))
            }
        };
        Self::with(api)
    }

    fn with(api: Result<Api, String>) -> Arc<Self> {
        let source = Arc::new(Self {
            api,
            cache: Mutex::new(HashMap::new()),
        });
        register_wipe(Arc::downgrade(&source) as Weak<dyn Wipe>);
        source
    }
}

#[async_trait]
impl SecretSource for ApiSource {
    fn plain_reveal(&self) -> bool {
        false
    }

    async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String> {
        let api = self.api.as_ref().map_err(Clone::clone)?;
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
        let wiped = !cache.is_empty();
        // Values overwrite themselves when dropped.
        cache.clear();
        if wiped {
            count_wiped();
        }
        wiped
    }
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

/// Reads the value of the secret `name` from the API.
#[cfg(feature = "real")]
async fn fetch(api: &Api, name: &str) -> Result<Zeroizing<Vec<u8>>, String> {
    use async_compat::Compat;

    let url = format!(
        "{}/execution/run/{}/secret/{name}",
        api.url.trim_end_matches('/'),
        api.run_id
    );
    let request = crate::http::CLIENT
        .as_ref()
        .map_err(Clone::clone)?
        .get(url)
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
        // The API gives the reason of a refusal, never a value. Proxies in between can
        // answer whole pages instead.
        let mut reason = String::from_utf8_lossy(&content)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if let Some((cut, _)) = reason.char_indices().nth(MAX_REASON_LENGTH) {
            reason.truncate(cut);
            reason.push('…');
        }
        if reason.is_empty() {
            Err(format!("Cadence.CI answered {status}"))
        } else {
            Err(format!("Cadence.CI answered {status}: {reason}"))
        }
    }
}

#[cfg(feature = "mock")]
async fn fetch(_api: &Api, name: &str) -> Result<Zeroizing<Vec<u8>>, String> {
    Err(format!(
        "secret '{name}' cannot be read from Cadence.CI in this build"
    ))
}
