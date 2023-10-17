use async_std::channel::{bounded, Sender};
use async_std::io::ReadExt;
use async_std::sync::{Arc, RwLock};
use melodium_core::common::executive::{Output, ResultStatus};
use melodium_core::*;
use melodium_macro::{check, mel_context, mel_model, mel_treatment};
use std::collections::HashMap;
use std::sync::Weak;
use tide::{Request, Response, Result, Server};

/// Describes HTTP request data.
///
/// - `id`: Identifier of connection, it is an arbitrary number that uniquely identifies a HTTP connection to a server during the duration it exists.
/// - `route`: The route called by the query.
/// - `uri`: The URI called by the query.
#[mel_context]
pub struct HttpRequest {
    pub id: u64,
    pub route: string,
    pub uri: string,
}

/// A HTTP server for general use.
///
/// The HTTP server provides configuration for receiving and responding to HTTP incoming requests.
/// - `bind`: The network address and port to listen, under the form `<ip/name>:<port>`.
/// - `routes`: The list of routes the server manages, usually at least composed of [`"/"`].
///
/// `HttpServer` aims to be used with `connection` treatment.
/// Every time a new HTTP request matching a configured route comes, a new track is created with `@HttpRequest` context.
///
/// ℹ️ If server binding fails, `failed_binding` is emitted.
///
/// ⚠️ Using `HttpServer` directly with `incoming` source and `outgoing` treatment should be done carefully.
///
#[mel_model(
    param routes Vec<string> none
    param bind string none
    source incoming (HttpRequest) (
        data Stream<byte>
        success Block<void>
        failure Block<void>
    )
    source failed_binding () (
        failure Block<void>
        error Block<string>
    )
    continuous (continuous)
)]
#[derive(Debug)]
pub struct HttpServer {
    model: Weak<HttpServerModel>,
    connections: Arc<RwLock<HashMap<u64, Sender<(u16, Vec<u8>)>>>>,
}

impl HttpServer {
    fn new(model: Weak<HttpServerModel>) -> Self {
        Self {
            model,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub(crate) fn connections(&self) -> Arc<RwLock<HashMap<u64, Sender<(u16, Vec<u8>)>>>> {
        Arc::clone(&self.connections)
    }

    async fn continuous(&self) {
        let mut server = Server::with_state(self.connections.clone());
        let model = self.model.upgrade().unwrap();
        for route in model.get_routes() {
            let model = Arc::clone(&model);
            server
                .at(&route)
                .all(move |request| Self::request(Arc::clone(&model), request, route.clone()));
        }
        match server.listen(model.get_bind()).await {
            Err(err) => {
                model
                    .new_failed_binding(
                        None,
                        Some(Box::new(|mut outputs| {
                            let error = outputs.get("error");
                            let failure = outputs.get("failure");

                            vec![Box::new(Box::pin(async move {
                                let _ = error.send_one_string(err.to_string()).await;
                                let _ = failure.send_one_void(()).await;
                                error.close().await;
                                failure.close().await;
                                ResultStatus::Ok
                            }))]
                        })),
                    )
                    .await
            }
            _ => {}
        }
    }

    async fn request(
        model: Arc<HttpServerModel>,
        request: Request<Arc<RwLock<HashMap<u64, Sender<(u16, Vec<u8>)>>>>>,
        route: String,
    ) -> Result {
        let (sender, receiver) = bounded(1);

        let id;
        {
            let mut lock = request.state().write().await;
            id = lock.len() as u64;
            lock.insert(id, sender);
        }

        let http_request = HttpRequest {
            id,
            route,
            uri: request.url().to_string(),
        };

        model
            .new_incoming(
                None,
                http_request,
                Some(Box::new(|mut outputs| {
                    let data = outputs.get("data");
                    let success = outputs.get("success");
                    let failure = outputs.get("failure");

                    vec![Box::new(Box::pin(Self::read_body(
                        request, data, success, failure,
                    )))]
                })),
            )
            .await;

        // TODO build a decent response, probably add status code and headers.
        match receiver.recv().await {
            Ok((status, data)) => Ok({
                let mut response = Response::new(status);
                response.set_body(data);
                response
            }),
            Err(_err) => Err(tide::Error::from_str(500, "")),
        }
    }

    async fn read_body(
        mut request: Request<Arc<RwLock<HashMap<u64, Sender<(u16, Vec<u8>)>>>>>,
        data: Box<dyn Output>,
        success: Box<dyn Output>,
        failure: Box<dyn Output>,
    ) -> ResultStatus {
        let mut body = request.take_body();
        loop {
            let mut buffer = vec![0; 2usize.pow(20)];
            match body.read(&mut buffer).await {
                Ok(0) => {
                    let _ = success.send_one_void(()).await;
                    break;
                }
                Ok(n) => {
                    buffer.truncate(n);
                    check!(data.send_byte(buffer).await);
                }
                Err(_err) => {
                    let _ = failure.send_one_void(()).await;
                    break;
                }
            }
        }

        data.close().await;
        success.close().await;
        failure.close().await;

        ResultStatus::Ok
    }
}

#[mel_treatment(
    input status Block<u16>
    input data Stream<byte>
    model http_server HttpServer
)]
pub async fn outgoing(id: u64) {
    let output;
    {
        let connections = HttpServerModel::into(http_server).inner().connections();
        let lock = connections.read().await;
        output = lock.get(&id).cloned();
    }
    if let Some(output) = output {
        let mut buffer = Vec::new();
        while let (Ok(data), false) = (data.recv_byte().await, output.is_closed()) {
            buffer.extend(data);
        }
        if let Ok(status) = status.recv_one_u16().await {
            let _ = output.send((status, buffer)).await;
        }
    }
}
