use crate::method::*;
use crate::status::*;
use async_ringbuf::AsyncHeapRb;
use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::{Arc, Weak};
use trillium_async_std::ClientConfig;
use trillium_client::Url;
use trillium_client::{Body, Client};
#[cfg(any(target_env = "msvc", target_vendor = "apple"))]
use trillium_native_tls::NativeTlsConfig as TlsConfig;
#[cfg(all(not(target_env = "msvc"), not(target_vendor = "apple")))]
use trillium_rustls::RustlsConfig as TlsConfig;

/// HTTP client for general use
///
/// The HTTP client provides configuration for HTTP requests.
///
/// - `base_url`: The base URL for a client. All request URLs will be relative to this URL.
/// - `tcp_no_delay`: TCP `NO_DELAY` field.
#[mel_model(
    param base_url Option<string> none
    param tcp_no_delay bool true
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

        let mut client = Client::new(config).with_default_pool();
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
///
/// - `status`: HTTP status response.
/// - `data`: data received as response, corresponding to the HTTP body.
/// - `failure`: emitted if the request failed technically, containing the failure message.
#[mel_treatment(
    model client HttpClient
    input url Block<string>
    output data Stream<byte>
    output failure Block<string>
    output status Block<HttpStatus>
)]
pub async fn request(method: HttpMethod) {
    if let Ok(url) = url
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        if let Some(client) = HttpClientModel::into(client).inner().client() {
            match client
                .base()
                .map(|base_url| base_url.join(&url))
                .unwrap_or_else(|| Url::parse(&url))
            {
                Ok(_) => match client.build_conn(method.0, url).await {
                    Ok(mut conn) => {
                        if let Some(recv_status) = conn.status() {
                            let _ = status
                                .send_one(Value::Data(
                                    Arc::new(HttpStatus(recv_status)) as Arc<dyn Data>
                                ))
                                .await;

                            let data_buf = AsyncHeapRb::<u8>::new(2usize.pow(20));
                            let (prod, mut cons) = data_buf.split();

                            let response_body = conn.response_body();
                            let _ =
                                futures::join!(async_std::io::copy(response_body, prod), async {
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
                                });
                        }
                    }
                    Err(err) => {
                        let _ = failure.send_one(err.to_string().into()).await;
                    }
                },
                Err(err) => {
                    let _ = failure.send_one(err.to_string().into()).await;
                }
            }
        }
    }
}

/// Performs HTTP operation with data emission.
///
/// This treatment process HTTP request to the given `url`.
/// - `method`: HTTP method used for the request.
///
/// - `url`: the URL to use for the request (combined with optionnal base from the client model), request starts as soon as the URL is transmitted.
/// - `body`: data to send as request body.
///
/// - `status`: HTTP status response.
/// - `data`: data received as response, corresponding to the HTTP body.
/// - `failure`: emitted if the request failed technically, containing the failure message.
#[mel_treatment(
    model client HttpClient
    input url Block<string>
    input body Stream<byte>
    output data Stream<byte>
    output failure Block<string>
    output status Block<HttpStatus>
)]
pub async fn request_with_body(method: HttpMethod) {
    if let Ok(url) = url
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        if let Some(client) = HttpClientModel::into(client).inner().client() {
            match client
                .base()
                .map(|base_url| base_url.join(&url))
                .unwrap_or_else(|| Url::parse(&url))
            {
                Ok(_) => {
                    let in_body_buf = AsyncHeapRb::<u8>::new(2usize.pow(20));
                    let (mut in_prod, in_cons) = in_body_buf.split();

                    let conn_doing = async {
                        client
                            .build_conn(method.0, url)
                            .with_body(Body::new_streaming(in_cons, None))
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
                    };

                    match futures::join!(body_transmission, conn_doing) {
                        (_, Ok(mut conn)) => {
                            if let Some(recv_status) = conn.status() {
                                let _ = status
                                    .send_one(Value::Data(
                                        Arc::new(HttpStatus(recv_status)) as Arc<dyn Data>
                                    ))
                                    .await;

                                let out_data_buf = AsyncHeapRb::<u8>::new(2usize.pow(20));
                                let (out_prod, mut out_cons) = out_data_buf.split();

                                let response_body = conn.response_body();
                                let _ = futures::join!(
                                    async_std::io::copy(response_body, out_prod),
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
                            let _ = failure.send_one(err.to_string().into()).await;
                        }
                    }
                }
                Err(err) => {
                    let _ = failure.send_one(err.to_string().into()).await;
                }
            }
        }
    }
}
