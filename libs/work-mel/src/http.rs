//! Requests to the Mélodium API.

use async_compat::Compat;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use std::sync::LazyLock;

/// HTTP client, running on Tokio: its requests are sent within `Compat`.
pub(crate) static CLIENT: LazyLock<Result<Client, String>> = LazyLock::new(|| {
    Client::builder()
        .user_agent(crate::USER_AGENT)
        .build()
        .map_err(|err| err.to_string())
});

/// Response to a request, with its body as text.
pub struct Response {
    pub status: u16,
    reason: &'static str,
    pub body: String,
}

impl Response {
    /// Describes a response that is not the expected one.
    pub fn error(&self) -> String {
        if self.status < 400 {
            format!("Server {} response: {}", self.status, self.body)
        } else if self.body.is_empty() {
            format!("API {} error: {}", self.status, self.reason)
        } else {
            format!("API {} error: {}: {}", self.status, self.reason, self.body)
        }
    }

    /// Gives the body, read as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, String> {
        serde_json::from_str(&self.body).map_err(|err| err.to_string())
    }
}

/// Sends `json` to `url` with POST, authorized by `authorization`.
pub async fn post(url: &str, authorization: &str, json: String) -> Result<Response, String> {
    let request = client()?
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .body(json);
    send(request, authorization).await
}

/// Gets `url`, authorized by `authorization`.
pub async fn get(url: &str, authorization: &str) -> Result<Response, String> {
    send(client()?.get(url), authorization).await
}

fn client() -> Result<&'static Client, String> {
    CLIENT.as_ref().map_err(Clone::clone)
}

async fn send(request: RequestBuilder, authorization: &str) -> Result<Response, String> {
    let request = request.header(AUTHORIZATION, authorization);
    Compat::new(async {
        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;
        Ok(Response {
            status: status.as_u16(),
            reason: status.canonical_reason().unwrap_or(""),
            body,
        })
    })
    .await
    .map_err(|err: reqwest::Error| err.to_string())
}
