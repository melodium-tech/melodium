use melodium_core::*;
use melodium_macro::{check, mel_model, mel_treatment};
use surf::{Client, Config};
use surf::Url;
use core::convert::TryInto;
use core::time::Duration;
use std::sync::{RwLock, Arc, Weak};
use async_std::io::ReadExt;

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
        Self { model, client: RwLock::new(None) }
    }

    pub fn initialization(&self) {

        let model = self.model.upgrade().unwrap();

        let mut config = Config::new();
        if !model.get_base_url().is_empty() { if let Ok(url) = Url::parse(&model.get_base_url()){
            config = config.set_base_url(url);
        }};

        config = config.set_http_keep_alive(model.get_keep_alive()).set_max_connections_per_host(
            match model.get_max_connections_per_host() {
                0 => usize::MAX,
                _ => model.get_max_connections_per_host() as usize
            }
        ).set_tcp_no_delay(model.get_tcp_no_delay())
        .set_timeout(Some(Duration::from_secs(model.get_timeout())));

    let client = config.try_into().ok().map(Arc::new);

    *self.client.write().unwrap() = client;
    }

    fn client(&self) -> Option<Arc<Client>> {
        self.client.read().unwrap().clone()
    }


}

#[mel_treatment(
    model http_client HttpClient
    input url Block<string>
    output data Stream<byte>
    output error Stream<string>
)]
pub async fn get() {
    if let Some(client) = HttpClientModel::into(http_client).inner().client() {
        if let Ok(url) = url.recv_one_string().await {
            match Url::parse(&url) {
                Ok(url) => {
                    let request = surf::Request::new(surf::http::Method::Get, url);

                    match client.send(request).await {
                        Ok(mut response) => {
                            let mut vec_data = vec![0; 2_usize.pow(20)];
                            loop {
                                match response.read(&mut vec_data).await {
                                    Ok(len) => {
                                        vec_data.truncate(len);
                                        check!(data.send_byte(vec_data).await);
                                        vec_data = vec![0; 2_usize.pow(20)];
                                    },
                                    Err(err) => {
                                        let _ = error.send_one_string(err.to_string()).await;
                                    }
                                }
                            }
                        },
                        Err(err) => {let _ = error.send_one_string(err.to_string()).await;}
                    }
                },
                Err(err) => {let _ = error.send_one_string(err.to_string()).await;}
            }
        }
    } else {
        let _ = error.send_one_string("No client available".to_string()).await;
    }
}
