//! The API, as reached by the run an engine is part of on Cadence.CI workers.

use uuid::Uuid;
use zeroize::Zeroizing;

/// Longest reason of a refusal given in errors, in characters.
#[cfg(feature = "real")]
const MAX_REASON_LENGTH: usize = 300;

/// How the run reaches the API: at `MELODIUM_API_URL`, with the token `MELODIUM_API_TOKEN` of
/// the run `MELODIUM_RUN_ID`. The token configures Mélodium itself, programs never get it.
pub(crate) struct RunApi {
    pub url: String,
    pub token: Zeroizing<String>,
    pub run_id: Uuid,
}

impl RunApi {
    /// Reaches the API as configured on Cadence.CI workers, or tells what is missing.
    pub fn from_environment() -> Result<Self, String> {
        let url = std::env::var("MELODIUM_API_URL").ok();
        let token = std::env::var("MELODIUM_API_TOKEN").ok().map(Zeroizing::new);
        let run_id = std::env::var("MELODIUM_RUN_ID").ok();
        match (url, token, run_id) {
            (Some(url), Some(token), Some(run_id)) => Uuid::parse_str(&run_id)
                .map(|run_id| RunApi { url, token, run_id })
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

    /// URL of `path` in the API.
    #[cfg(feature = "real")]
    pub fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.url.trim_end_matches('/'))
    }

    /// URL of `path` among the endpoints of the run.
    #[cfg(feature = "real")]
    pub fn run_url(&self, path: &str) -> String {
        self.url(&format!("execution/run/{}/{path}", self.run_id))
    }

    /// Sends `body` as JSON to `url` with POST, giving the status and body of the answer.
    #[cfg(feature = "real")]
    pub async fn post_json(
        &self,
        url: String,
        body: &impl serde::Serialize,
    ) -> Result<(reqwest::StatusCode, bytes::Bytes), String> {
        let body = serde_json::to_vec(body).map_err(|err| err.to_string())?;
        let request = crate::http::CLIENT
            .as_ref()
            .map_err(Clone::clone)?
            .post(url)
            .bearer_auth(self.token.as_str())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body);
        Self::send(request).await
    }

    #[cfg(feature = "real")]
    async fn send(
        request: reqwest::RequestBuilder,
    ) -> Result<(reqwest::StatusCode, bytes::Bytes), String> {
        // The HTTP client runs on Tokio.
        async_compat::Compat::new(async {
            let response = request.send().await?;
            let status = response.status();
            response.bytes().await.map(|body| (status, body))
        })
        .await
        .map_err(|err| format!("Cadence.CI cannot be reached: {err}"))
    }
}

/// Reads the JSON answer of the API, or describes its refusal.
#[cfg(feature = "real")]
pub(crate) fn answer<T: serde::de::DeserializeOwned>(
    (status, body): (reqwest::StatusCode, bytes::Bytes),
) -> Result<T, String> {
    if status.is_success() {
        serde_json::from_slice(&body)
            .map_err(|err| format!("Cadence.CI gave an unexpected answer: {err}"))
    } else {
        Err(refusal(status, &body))
    }
}

/// Describes a refusal of the API, from its status and body.
#[cfg(feature = "real")]
pub(crate) fn refusal(status: reqwest::StatusCode, body: &[u8]) -> String {
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
