use crate::method::*;
use crate::status::*;
use async_ringbuf::AsyncHeapRb;
use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::{Arc, Weak};
use std_mel::data::string_map::*;
use trillium::HeaderName;
use trillium::HeaderValue;
use trillium::KnownHeaderName;
use trillium_async_std::ClientConfig;
use trillium_client::Url;
use trillium_client::{Body, Client};
/*#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
use trillium_native_tls::NativeTlsConfig as TlsConfig;
#[cfg(all(not(target_env = "msvc"), not(target_vendor = "apple")))]*/
use trillium_rustls::RustlsConfig as TlsConfig;

pub const USER_AGENT: &str = concat!("http-mel/", env!("CARGO_PKG_VERSION"));

/// HTTP client for general use
///
/// The HTTP client provides configuration for HTTP requests.
///
/// - `base_url`: The base URL for a client. All request URLs will be relative to this URL.
/// - `tcp_no_delay`: TCP `NO_DELAY` field.
/// - `headers`: Headers to add in requests made with this client.
///
/// The default headers are `Accept: */*` and `User-Agent: http-mel/<version>`
#[mel_model(
    param base_url Option<string> none
    param tcp_no_delay bool true
    param headers StringMap none
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

    fn initialization(&self) {
        let model = self.model.upgrade().unwrap();

        let config = TlsConfig::default()
            .with_tcp_config(ClientConfig::new().with_nodelay(model.get_tcp_no_delay()));

        let mut client = Client::new(config)
            .with_default_pool()
            .with_default_header(KnownHeaderName::UserAgent, USER_AGENT);
        if let Some(base) = model.get_base_url() {
            if let Ok(url) = Url::parse(&base) {
                client = client.with_base(url);
            }
        }

        *self.client.write().unwrap() = Some(Arc::new(client));
    }

    fn client(&self) -> Option<Arc<Client>> {
        self.client.read().unwrap().clone()
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}
}

/// Performs HTTP operation without data emission.
///
/// This treatment process HTTP request to the given `url`.
/// - `method`: HTTP method used for the request.
///
/// - `url`: the URL to use for the request (combined with optionnal base from the client model), request starts as soon as the URL is transmitted.
/// - `req_headers`: the headers to use for the request (combined with ones defined at client level).
///
/// - `status`: HTTP status response.
/// - `res_headers`: the headers contained in the response.
/// - `data`: data received as response, corresponding to the HTTP body.
/// - `completed`: emitted when the incoming request finished successfully.
/// - `failed`: emitted if the incoming request failed technically.
/// - `error`: message containing error when request failed technically.
/// - `finished`: emitted when the incoming request finished, regardless of state.
#[mel_treatment(
    model client HttpClient
    input url Block<string>
    input req_headers Block<StringMap>
    output res_headers Block<StringMap>
    output data Stream<byte>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output error Block<string>
    output status Block<HttpStatus>
)]
pub async fn request(method: HttpMethod) {
    if let (Ok(url), Ok(req_headers)) = (
        url.recv_one()
            .await
            .map(|val| GetData::<string>::try_data(val).unwrap()),
        req_headers.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<StringMap>()
                .unwrap()
        }),
    ) {
        if let Some(client) = HttpClientModel::into(client).inner().client() {
            match client
                .base()
                .map(|base_url| base_url.join(&url))
                .unwrap_or_else(|| Url::parse(&url))
            {
                Ok(url) => match {
                    let mut conn = client.build_conn(method.0, url);
                    for (name, content) in &req_headers.map {
                        let header_name = HeaderName::from(name.to_string());
                        if header_name.is_valid() {
                            let header_content = HeaderValue::from(content.clone());
                            if header_content.is_valid() {
                                conn.request_headers_mut()
                                    .insert(header_name.to_owned(), header_content);
                            }
                        }
                    }
                    conn
                }
                .await
                {
                    Ok(mut conn) => {
                        if let Some(recv_status) = conn.status() {
                            let _ = status
                                .send_one(Value::Data(
                                    Arc::new(HttpStatus(recv_status)) as Arc<dyn Data>
                                ))
                                .await;

                            let headers = conn
                                .response_headers()
                                .iter()
                                .filter_map(|(name, value)| {
                                    value
                                        .as_str()
                                        .map(|value| (name.to_string(), value.to_string()))
                                })
                                .collect();
                            let _ = res_headers
                                .send_one(Value::Data(
                                    Arc::new(StringMap::new_with(headers)) as Arc<dyn Data>
                                ))
                                .await;

                            status.close().await;
                            res_headers.close().await;

                            let data_buf = AsyncHeapRb::<u8>::new(2usize.pow(20));
                            let (prod, mut cons) = data_buf.split();

                            let response_body = conn.response_body();
                            let _ = futures::join!(
                                async {
                                    let _ = async_std::io::copy(response_body, prod).await;
                                    let _ = completed.send_one(().into()).await;
                                },
                                async {
                                    loop {
                                        let mut size = 2usize.pow(20);
                                        let mut recv_data = vec![0; size];

                                        match cons.pop_slice(&mut recv_data).await {
                                            Ok(_) => {}
                                            Err(written_size) => size = written_size,
                                        }

                                        recv_data.truncate(size);

                                        check!(
                                            data.send_many(TransmissionValue::Byte(
                                                recv_data.into()
                                            ))
                                            .await
                                        );
                                        if cons.is_closed() {
                                            break;
                                        }
                                    }
                                }
                            );
                        }
                    }
                    Err(err) => {
                        let _ = failed.send_one(().into()).await;
                        let _ = error.send_one(err.to_string().into()).await;
                    }
                },
                Err(err) => {
                    let _ = failed.send_one(().into()).await;
                    let _ = error.send_one(err.to_string().into()).await;
                }
            }
            let _ = finished.send_one(().into()).await;
        }
    }
}

/// Performs HTTP operation with data emission.
///
/// This treatment process HTTP request to the given `url`.
/// - `method`: HTTP method used for the request.
///
/// - `url`: the URL to use for the request (combined with optionnal base from the client model), request starts as soon as the URL is transmitted.
/// - `req_headers`: the headers to use for the request (combined with ones defined at client level).
/// - `body`: data to send as request body.
///
/// - `status`: HTTP status response.
/// - `res_headers`: the headers contained in the response.
/// - `data`: data received as response, corresponding to the HTTP body.
/// - `completed`: emitted when the request finished successfully.
/// - `failed`: emitted if the request failed technically.
/// - `error`: message containing error when request failed technically.
/// - `finished`: emitted when the request finished, regardless of state.
#[mel_treatment(
    model client HttpClient
    input url Block<string>
    input req_headers Block<StringMap>
    input body Stream<byte>
    output data Stream<byte>
    output res_headers Block<StringMap>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output error Block<string>
    output status Block<HttpStatus>
)]
pub async fn request_with_body(method: HttpMethod) {
    if let (Ok(url), Ok(req_headers)) = (
        url.recv_one()
            .await
            .map(|val| GetData::<string>::try_data(val).unwrap()),
        req_headers.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<StringMap>()
                .unwrap()
        }),
    ) {
        if let Some(client) = HttpClientModel::into(client).inner().client() {
            match client
                .base()
                .map(|base_url| base_url.join(&url))
                .unwrap_or_else(|| Url::parse(&url))
            {
                Ok(url) => {
                    let in_body_buf = AsyncHeapRb::<u8>::new(2usize.pow(20));
                    let (mut in_prod, in_cons) = in_body_buf.split();

                    let conn_doing = async {
                        {
                            let mut conn = client.build_conn(method.0, url);

                            for (name, content) in &req_headers.map {
                                let header_name = HeaderName::from(name.to_string());
                                if header_name.is_valid() {
                                    let header_content = HeaderValue::from(content.to_string());
                                    if header_content.is_valid() {
                                        conn.request_headers_mut()
                                            .insert(header_name.to_owned(), header_content);
                                    }
                                }
                            }
                            conn.with_body(Body::new_streaming(in_cons, None))
                        }
                        .await
                    };
                    let body_transmission = async {
                        while let Ok(body_data) = body
                            .recv_many()
                            .await
                            .map(|values| TryInto::<VecDeque<u8>>::try_into(values).unwrap())
                        {
                            if let Err(_) = in_prod.push_iter(body_data.into_iter()).await {
                                break;
                            }
                        }
                        in_prod.close();
                    };

                    match futures::join!(body_transmission, conn_doing) {
                        (_, Ok(mut conn)) => {
                            if let Some(recv_status) = conn.status() {
                                let _ = status
                                    .send_one(Value::Data(
                                        Arc::new(HttpStatus(recv_status)) as Arc<dyn Data>
                                    ))
                                    .await;

                                let headers = conn
                                    .response_headers()
                                    .iter()
                                    .filter_map(|(name, value)| {
                                        value
                                            .as_str()
                                            .map(|value| (name.to_string(), value.to_string()))
                                    })
                                    .collect();
                                let _ = res_headers
                                    .send_one(Value::Data(
                                        Arc::new(StringMap::new_with(headers)) as Arc<dyn Data>
                                    ))
                                    .await;

                                status.close().await;
                                res_headers.close().await;

                                let out_data_buf = AsyncHeapRb::<u8>::new(2usize.pow(20));
                                let (out_prod, mut out_cons) = out_data_buf.split();

                                let response_body = conn.response_body();
                                let _ = futures::join!(
                                    async {
                                        let _ = async_std::io::copy(response_body, out_prod).await;
                                        let _ = completed.send_one(().into()).await;
                                    },
                                    async {
                                        loop {
                                            let mut size = 2usize.pow(20);
                                            let mut recv_data = vec![0; size];
                                            match out_cons.pop_slice(&mut recv_data).await {
                                                Ok(_) => {}
                                                Err(written_size) => size = written_size,
                                            }

                                            recv_data.truncate(size);

                                            check!(
                                                data.send_many(TransmissionValue::Byte(
                                                    recv_data.into()
                                                ))
                                                .await
                                            );
                                            if out_cons.is_closed() {
                                                break;
                                            }
                                        }
                                    }
                                );
                            }
                        }
                        (_, Err(err)) => {
                            let _ = failed.send_one(().into()).await;
                            let _ = error.send_one(err.to_string().into()).await;
                        }
                    }
                }
                Err(err) => {
                    let _ = failed.send_one(().into()).await;
                    let _ = error.send_one(err.to_string().into()).await;
                }
            }
            let _ = finished.send_one(().into()).await;
        }
    }
}
