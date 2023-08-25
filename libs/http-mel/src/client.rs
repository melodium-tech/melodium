use async_std::io::ReadExt;
use core::convert::TryInto;
use core::time::Duration;
use melodium_core::common::executive::Output;
use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::sync::{Arc, RwLock, Weak};
use surf::Url;
use surf::{Client, Config};

/// HTTP client for general use
///
/// The HTTP client provides configuration for HTTP requests.
/// All the default parameters are set up as good trade-off for general use.
///
/// - `base_url`: The base URL for a client. All request URLs will be relative to this URL. ℹ️ Note: a trailing slash is significant. Without it, the last path component is considered to be a “file” name to be removed to get at the “directory” that is used as the base.
/// - `keep_alive`: HTTP 1.1 `keep-alive`, for connection pooling.
/// - `max_connections_per_host`: Maximum number of simultaneous connections that the client is allowed to keep open to individual hosts at one time.
/// - `tcp_no_delay`: TCP `NO_DELAY` field.
/// - `timeout`: Connection timeout duration.
#[mel_model(
    param base_url string ""
    param keep_alive bool true
    param max_connections_per_host u64 50
    param tcp_no_delay bool false
    param timeout u64 60
    initialize initialization
)]
#[derive(Debug)]
pub struct HttpClient {
    model: Weak<HttpClientModel>,
    client: RwLock<Option<Arc<Client>>>,
}

impl HttpClient {
    fn new(model: Weak<HttpClientModel>) -> Self {
        Self {
            model,
            client: RwLock::new(None),
        }
    }

    pub fn initialization(&self) {
        let model = self.model.upgrade().unwrap();

        let mut config = Config::new();
        if !model.get_base_url().is_empty() {
            if let Ok(url) = Url::parse(&model.get_base_url()) {
                config = config.set_base_url(url);
            }
        };

        config = config
            .set_http_keep_alive(model.get_keep_alive())
            .set_max_connections_per_host(match model.get_max_connections_per_host() {
                0 => usize::MAX,
                _ => model.get_max_connections_per_host() as usize,
            })
            .set_tcp_no_delay(model.get_tcp_no_delay())
            .set_timeout(Some(Duration::from_secs(model.get_timeout())));

        let client = config.try_into().ok().map(Arc::new);

        *self.client.write().unwrap() = client;
    }

    fn client(&self) -> Option<Arc<Client>> {
        self.client.read().unwrap().clone()
    }

    pub async fn manage_request(
        &self,
        request: surf::Request,
        data: Option<&dyn Output>,
        failure: &dyn Output,
        is_error: &dyn Output,
        is_success: &dyn Output,
        http_code: &dyn Output,
        http_status: &dyn Output,
    ) {
        if let Some(client) = self.client() {
            match client.send(request).await {
                Ok(mut response) => {
                    let status = response.status();
                    if status.is_client_error() || status.is_server_error() {
                        let _ = is_error.send_one_bool(true).await;
                        let _ = is_success.send_one_bool(false).await;
                    } else {
                        let _ = is_error.send_one_bool(false).await;
                        let _ = is_success.send_one_bool(true).await;
                    }

                    let _ = http_code.send_one_u16(status as u16).await;
                    let _ = http_status
                        .send_one_string(status.canonical_reason().to_string())
                        .await;

                    if let Some(data) = data {
                        let mut vec_data = vec![0; 2_usize.pow(20)];
                        loop {
                            match response.read(&mut vec_data).await {
                                Ok(len) if len > 0 => {
                                    vec_data.truncate(len);
                                    check!(data.send_byte(vec_data).await);
                                    vec_data = vec![0; 2_usize.pow(20)];
                                }
                                Ok(_) => break,
                                Err(err) => {
                                    let _ = failure.send_one_string(err.to_string()).await;
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    let _ = failure.send_one_string(err.to_string()).await;
                }
            }
        } else {
            let _ = failure
                .send_one_string("No HTTP client available".to_string())
                .await;
        }
    }
}

/// Performs HTTP DELETE operation.
///
/// `url` input gives the URL to call DELETE on. Request starts as soon as the URL is transmitted.
/// `failure` output gives the failures messages related to the network connection or HTTP transgression (and _not_ the errors that are defined by HTTP standard).
/// `is_success` tells that server responded with positive code (`1xx`, `2xx` and `3xx` status ranges).
/// `is_error` tells that server responded with negative code (`4xx` and `5xx` status ranges).
/// `is_success` and `is_error` are mutually exclusive.
///
/// `http_code` contains the response code.
/// `http_status` contains the canonical reason of the status code.
///
/// Also see [MDN documentation](https://developer.mozilla.org/docs/Web/HTTP/Methods/DELETE).
///
#[mel_treatment(
    model http_client HttpClient
    input url Block<string>
    output failure Stream<string>
    output is_error Block<bool>
    output is_success Block<bool>
    output http_code Block<u16>
    output http_status Block<string>
)]
pub async fn delete() {
    if let Ok(url) = url.recv_one_string().await {
        match Url::parse(&url) {
            Ok(url) => {
                let request = surf::Request::new(surf::http::Method::Delete, url);

                HttpClientModel::into(http_client)
                    .inner()
                    .manage_request(
                        request,
                        None,
                        &*failure,
                        &*is_error,
                        &*is_success,
                        &*http_code,
                        &*http_status,
                    )
                    .await;
            }
            Err(err) => {
                let _ = failure.send_one_string(err.to_string()).await;
            }
        }
    }
}

/// Performs HTTP GET operation.
///
/// `url` input gives the URL to call GET on. Request starts as soon as the URL is transmitted.
/// `failure` output gives the failures messages related to the network connection or HTTP transgression (and _not_ the errors that are defined by HTTP standard).
/// `is_success` tells that server responded with positive code (`1xx`, `2xx` and `3xx` status ranges).
/// `is_error` tells that server responded with negative code (`4xx` and `5xx` status ranges).
/// `is_success` and `is_error` are mutually exclusive.
///
/// `data` outputs the data body received from the server.
/// `http_code` contains the response code.
/// `http_status` contains the canonical reason of the status code.
///
/// Also see [MDN documentation](https://developer.mozilla.org/docs/Web/HTTP/Methods/GET).
///
#[mel_treatment(
    model http_client HttpClient
    input url Block<string>
    output data Stream<byte>
    output failure Stream<string>
    output is_error Block<bool>
    output is_success Block<bool>
    output http_code Block<u16>
    output http_status Block<string>
)]
pub async fn get() {
    if let Ok(url) = url.recv_one_string().await {
        match Url::parse(&url) {
            Ok(url) => {
                let request = surf::Request::new(surf::http::Method::Get, url);

                HttpClientModel::into(http_client)
                    .inner()
                    .manage_request(
                        request,
                        Some(&*data),
                        &*failure,
                        &*is_error,
                        &*is_success,
                        &*http_code,
                        &*http_status,
                    )
                    .await;
            }
            Err(err) => {
                let _ = failure.send_one_string(err.to_string()).await;
            }
        }
    }
}

/// Performs HTTP HEAD operation.
///
/// `url` input gives the URL to call HEAD on. Request starts as soon as the URL is transmitted.
/// `failure` output gives the failures messages related to the network connection or HTTP transgression (and _not_ the errors that are defined by HTTP standard).
/// `is_success` tells that server responded with positive code (`1xx`, `2xx` and `3xx` status ranges).
/// `is_error` tells that server responded with negative code (`4xx` and `5xx` status ranges).
/// `is_success` and `is_error` are mutually exclusive.
///
/// `http_code` contains the response code.
/// `http_status` contains the canonical reason of the status code.
///
/// Also see [MDN documentation](https://developer.mozilla.org/docs/Web/HTTP/Methods/HEAD).
///
#[mel_treatment(
    model http_client HttpClient
    input url Block<string>
    output failure Stream<string>
    output is_error Block<bool>
    output is_success Block<bool>
    output http_code Block<u16>
    output http_status Block<string>
)]
pub async fn head() {
    if let Ok(url) = url.recv_one_string().await {
        match Url::parse(&url) {
            Ok(url) => {
                let request = surf::Request::new(surf::http::Method::Head, url);

                HttpClientModel::into(http_client)
                    .inner()
                    .manage_request(
                        request,
                        None,
                        &*failure,
                        &*is_error,
                        &*is_success,
                        &*http_code,
                        &*http_status,
                    )
                    .await;
            }
            Err(err) => {
                let _ = failure.send_one_string(err.to_string()).await;
            }
        }
    }
}

/// Performs HTTP OPTIONS operation.
///
/// `url` input gives the URL to call OPTIONS on. Request starts as soon as the URL is transmitted.
/// `failure` output gives the failures messages related to the network connection or HTTP transgression (and _not_ the errors that are defined by HTTP standard).
/// `is_success` tells that server responded with positive code (`1xx`, `2xx` and `3xx` status ranges).
/// `is_error` tells that server responded with negative code (`4xx` and `5xx` status ranges).
/// `is_success` and `is_error` are mutually exclusive.
///
/// `data` outputs the data body received from the server.
/// `http_code` contains the response code.
/// `http_status` contains the canonical reason of the status code.
///
/// Also see [MDN documentation](https://developer.mozilla.org/docs/Web/HTTP/Methods/OPTIONS).
///
#[mel_treatment(
    model http_client HttpClient
    input url Block<string>
    output data Stream<byte>
    output failure Stream<string>
    output is_error Block<bool>
    output is_success Block<bool>
    output http_code Block<u16>
    output http_status Block<string>
)]
pub async fn options() {
    if let Ok(url) = url.recv_one_string().await {
        match Url::parse(&url) {
            Ok(url) => {
                let request = surf::Request::new(surf::http::Method::Options, url);

                HttpClientModel::into(http_client)
                    .inner()
                    .manage_request(
                        request,
                        Some(&*data),
                        &*failure,
                        &*is_error,
                        &*is_success,
                        &*http_code,
                        &*http_status,
                    )
                    .await;
            }
            Err(err) => {
                let _ = failure.send_one_string(err.to_string()).await;
            }
        }
    }
}

/// Performs HTTP PATCH operation.
///
/// `url` input gives the URL to call PATCH on. Request starts as soon as the URL is transmitted.
/// `data` input gives request content to send to the server.
///
/// `failure` output gives the failures messages related to the network connection or HTTP transgression (and _not_ the errors that are defined by HTTP standard).
/// `is_success` tells that server responded with positive code (`1xx`, `2xx` and `3xx` status ranges).
/// `is_error` tells that server responded with negative code (`4xx` and `5xx` status ranges).
/// `is_success` and `is_error` are mutually exclusive.
///
/// `http_code` contains the response code.
/// `http_status` contains the canonical reason of the status code.
///
/// Also see [MDN documentation](https://developer.mozilla.org/docs/Web/HTTP/Methods/PATCH).
///
#[mel_treatment(
    model http_client HttpClient
    input url Block<string>
    input data Block<Vec<byte>>
    output failure Stream<string>
    output is_error Block<bool>
    output is_success Block<bool>
    output http_code Block<u16>
    output http_status Block<string>
)]
pub async fn patch() {
    if let Ok(url) = url.recv_one_string().await {
        if let Ok(data) = data.recv_one_vec_byte().await {
            match Url::parse(&url) {
                Ok(url) => {
                    let mut request = surf::Request::new(surf::http::Method::Patch, url);

                    request.body_bytes(&data);

                    HttpClientModel::into(http_client)
                        .inner()
                        .manage_request(
                            request,
                            None,
                            &*failure,
                            &*is_error,
                            &*is_success,
                            &*http_code,
                            &*http_status,
                        )
                        .await;
                }
                Err(err) => {
                    let _ = failure.send_one_string(err.to_string()).await;
                }
            }
        }
    }
}

/// Performs HTTP POST operation.
///
/// `url` input gives the URL to call POST on. Request starts as soon as the URL is transmitted.
/// `mime` input gives request content type.
/// `form` input gives request content to send to the server.
///
/// `failure` output gives the failures messages related to the network connection or HTTP transgression (and _not_ the errors that are defined by HTTP standard).
/// `is_success` tells that server responded with positive code (`1xx`, `2xx` and `3xx` status ranges).
/// `is_error` tells that server responded with negative code (`4xx` and `5xx` status ranges).
/// `is_success` and `is_error` are mutually exclusive.
///
/// `data` outputs the data body received from the server.
/// `http_code` contains the response code.
/// `http_status` contains the canonical reason of the status code.
///
/// Also see [MDN documentation](https://developer.mozilla.org/docs/Web/HTTP/Methods/POST).
///
#[mel_treatment(
    model http_client HttpClient
    input url Block<string>
    input mime Block<string>
    input form Block<Vec<byte>>
    output data Stream<byte>
    output failure Stream<string>
    output is_error Block<bool>
    output is_success Block<bool>
    output http_code Block<u16>
    output http_status Block<string>
)]
pub async fn post() {
    if let Ok(url) = url.recv_one_string().await {
        if let Ok(mime) = mime.recv_one_string().await {
            if let Ok(form_data) = form.recv_one_vec_byte().await {
                match Url::parse(&url) {
                    Ok(url) => {
                        let mut request = surf::Request::new(surf::http::Method::Post, url);

                        request.body_bytes(&form_data);
                        request.set_content_type(surf::http::Mime::from(mime.as_str()));

                        HttpClientModel::into(http_client)
                            .inner()
                            .manage_request(
                                request,
                                Some(&*data),
                                &*failure,
                                &*is_error,
                                &*is_success,
                                &*http_code,
                                &*http_status,
                            )
                            .await;
                    }
                    Err(err) => {
                        let _ = failure.send_one_string(err.to_string()).await;
                    }
                }
            }
        }
    }
}

/// Performs HTTP PUT operation.
///
/// `url` input gives the URL to call PUT on. Request starts as soon as the URL is transmitted.
/// `mime` input gives request content type.
/// `data` input gives request content to send to the server.
///
/// `failure` output gives the failures messages related to the network connection or HTTP transgression (and _not_ the errors that are defined by HTTP standard).
/// `is_success` tells that server responded with positive code (`1xx`, `2xx` and `3xx` status ranges).
/// `is_error` tells that server responded with negative code (`4xx` and `5xx` status ranges).
/// `is_success` and `is_error` are mutually exclusive.
///
/// `http_code` contains the response code.
/// `http_status` contains the canonical reason of the status code.
///
/// Also see [MDN documentation](https://developer.mozilla.org/docs/Web/HTTP/Methods/PUT).
///
#[mel_treatment(
    model http_client HttpClient
    input url Block<string>
    input mime Block<string>
    input data Block<Vec<byte>>
    output failure Stream<string>
    output is_error Block<bool>
    output is_success Block<bool>
    output http_code Block<u16>
    output http_status Block<string>
)]
pub async fn put() {
    if let Ok(url) = url.recv_one_string().await {
        if let Ok(mime) = mime.recv_one_string().await {
            if let Ok(data) = data.recv_one_vec_byte().await {
                match Url::parse(&url) {
                    Ok(url) => {
                        let mut request = surf::Request::new(surf::http::Method::Put, url);

                        request.body_bytes(&data);
                        request.set_content_type(surf::http::Mime::from(mime.as_str()));

                        HttpClientModel::into(http_client)
                            .inner()
                            .manage_request(
                                request,
                                None,
                                &*failure,
                                &*is_error,
                                &*is_success,
                                &*http_code,
                                &*http_status,
                            )
                            .await;
                    }
                    Err(err) => {
                        let _ = failure.send_one_string(err.to_string()).await;
                    }
                }
            }
        }
    }
}

/// Performs HTTP TRACE operation.
///
/// `url` input gives the URL to call TRACE on. Request starts as soon as the URL is transmitted.
/// `failure` output gives the failures messages related to the network connection or HTTP transgression (and _not_ the errors that are defined by HTTP standard).
/// `is_success` tells that server responded with positive code (`1xx`, `2xx` and `3xx` status ranges).
/// `is_error` tells that server responded with negative code (`4xx` and `5xx` status ranges).
/// `is_success` and `is_error` are mutually exclusive.
///
/// `http_code` contains the response code.
/// `http_status` contains the canonical reason of the status code.
///
/// Also see [MDN documentation](https://developer.mozilla.org/docs/Web/HTTP/Methods/TRACE).
///
#[mel_treatment(
    model http_client HttpClient
    input url Block<string>
    output failure Stream<string>
    output is_error Block<bool>
    output is_success Block<bool>
    output http_code Block<u16>
    output http_status Block<string>
)]
pub async fn trace() {
    if let Ok(url) = url.recv_one_string().await {
        match Url::parse(&url) {
            Ok(url) => {
                let request = surf::Request::new(surf::http::Method::Trace, url);

                HttpClientModel::into(http_client)
                    .inner()
                    .manage_request(
                        request,
                        None,
                        &*failure,
                        &*is_error,
                        &*is_success,
                        &*http_code,
                        &*http_status,
                    )
                    .await;
            }
            Err(err) => {
                let _ = failure.send_one_string(err.to_string()).await;
            }
        }
    }
}
