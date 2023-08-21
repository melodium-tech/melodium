use async_std::channel::{bounded, Sender};
use async_std::sync::{Arc, RwLock};
use melodium_core::common::executive::{Output, ResultStatus};
use melodium_core::*;
use melodium_macro::{check, mel_context, mel_model, mel_treatment};
use std::collections::HashMap;
use std::sync::Weak;
use tide::{Request, Response, Result, Server};

#[mel_context]
pub struct HttpRequest {
    pub id: u64,
    pub route: string,
    pub uri: string,
}

#[mel_model(
    param routes Vec<string> none
    source incoming (HttpRequest) (data Stream<byte>)
    continuous (continuous)
)]
#[derive(Debug)]
pub struct HttpServer {
    model: Weak<HttpServerModel>,
    connections: Arc<RwLock<HashMap<u64, Sender<Vec<u8>>>>>,
}

impl HttpServer {
    fn new(model: Weak<HttpServerModel>) -> Self {
        Self {
            model,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn continuous(&self) {
        let mut server = Server::with_state(self.connections.clone());
        for route in self.model.upgrade().unwrap().get_routes() {
            server
                .at(&route)
                .all(|request| self.request(request, route.clone()));
        }
    }

    async fn request(
        &self,
        request: Request<Arc<RwLock<HashMap<u64, Sender<Vec<u8>>>>>>,
        route: String,
    ) -> Result {
        let (sender, receiver) = bounded(1);

        let id;
        {
            let mut lock = self.connections.write().await;
            id = lock.len() as u64;
            lock.insert(id, sender);
        }

        let http_request = HttpRequest {
            id,
            route,
            uri: request.url().to_string(),
        };

        self.model.upgrade().unwrap().new_incoming(
            None,
            http_request,
            Some(Box::new(|mut outputs| {
                let data = outputs.remove("data").unwrap();

                vec![Box::new(Box::pin(Self::read_body(request, data)))]
            })),
        );

        // TODO build a decent response, probably add status code and headers.
        match receiver.recv().await {
            Ok(data) => Ok({
                let mut response = Response::new(200);
                response.set_body(data);
                response
            }),
            Err(err) => Err(tide::Error::from_str(500, "")),
        }
    }

    async fn read_body(
        mut request: Request<Arc<RwLock<HashMap<u64, Sender<Vec<u8>>>>>>,
        data: Box<dyn Output>,
    ) -> ResultStatus {
        // TODO fix the reading to send failure information when body_bytes (or better method) fails.
        let _ = data
            .send_byte(request.body_bytes().await.unwrap_or_default())
            .await;
        ResultStatus::Ok
    }
}
