use crate::method::*;
use crate::status::*;
use async_ringbuf::{AsyncHeapRb, AsyncProducer, AsyncRb};
use async_std::sync::{Arc as AsyncArc, Barrier as AsyncBarrier, RwLock as AsyncRwLock};
use core::{fmt::Debug, mem::MaybeUninit};
use melodium_core::{common::executive::ResultStatus, *};
use melodium_macro::{mel_context, mel_model, mel_treatment};
use net_mel::ip::*;
use ringbuf::SharedRb;
use routefinder::RouteSpec;
use routefinder::Segment;
use std::sync::Arc;
use std::{
    collections::HashMap,
    sync::{RwLock, Weak},
};
use std_mel::data::*;
use trillium::HeaderName;
use trillium::HeaderValue;
use trillium::KnownHeaderName;
use trillium::{Body, Conn};
use trillium::{Method, Status};
use trillium_async_std::Stopper;
use trillium_router::{Router, RouterConnExt};
use uuid::Uuid;

pub const SERVER: &str = concat!("http-mel/", env!("CARGO_PKG_VERSION"));

/// Describes HTTP request data.
///
/// - `id`: identifier of connection, it is an arbitrary number that uniquely identifies a HTTP connection to a server.
/// - `route`: the route used by the request.
/// - `path`: the path called by the request.
/// - `parameters`: the parameters from the route.
/// - `method`: the HTTP method used by the request.
#[mel_context]
pub struct HttpRequest {
    pub id: u128,
    pub route: string,
    pub path: string,
    pub parameters: Map,
    pub method: HttpMethod,
}

type AsyncProducerStatus =
    AsyncProducer<Status, Arc<AsyncRb<Status, SharedRb<Status, Vec<MaybeUninit<Status>>>>>>;
type AsyncProducerHeaders =
    AsyncProducer<Map, Arc<AsyncRb<Map, SharedRb<Map, Vec<MaybeUninit<Map>>>>>>;
type AsyncProducerOutgoing =
    AsyncProducer<u8, Arc<AsyncRb<u8, SharedRb<u8, Vec<MaybeUninit<u8>>>>>>;

/// A HTTP server for general use.
///
/// The HTTP server provides configuration for receiving and responding to HTTP incoming requests.
/// - `host`: the network address to bind with.
/// - `port`: the port to bind with.
///
/// `HttpServer` aims to be used with `connection` treatment.
/// Every time a new HTTP request matching a configured route comes, a new track is created with `@HttpRequest` context.
///
/// ℹ️ If server binding fails, `failed_binding` is emitted.
///
/// ⚠️ Use `HttpServer` with `connection` treatment, as using `incoming` source and `outgoing` treatment directly should be done carefully.
///
#[mel_model(
    param host Ip none
    param port u16 none
    source incoming (HttpRequest) (
        param method HttpMethod none
        param route string none
    ) (
        started Block<void>
        headers Block<Map>
        data Stream<byte>
        failure Block<string>
    )
    source failed_binding () () (
        failure Block<string>
    )
    continuous (continuous)
    shutdown shutdown
)]
pub struct HttpServer {
    model: Weak<HttpServerModel>,
    launch_barrier: AsyncArc<AsyncBarrier>,
    routes: RwLock<Vec<(Arc<HttpMethod>, String)>>,
    status: AsyncArc<AsyncRwLock<HashMap<Uuid, AsyncProducerStatus>>>,
    headers: AsyncArc<AsyncRwLock<HashMap<Uuid, AsyncProducerHeaders>>>,
    outgoing: AsyncArc<AsyncRwLock<HashMap<Uuid, AsyncProducerOutgoing>>>,
    shutdown: Stopper,
}

impl Debug for HttpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpServer")
            .field("model", &self.model)
            .field("routes", &self.routes)
            .field("shutdown", &self.shutdown)
            .finish()
    }
}

impl HttpServer {
    pub fn new(model: Weak<HttpServerModel>) -> Self {
        Self {
            model,
            launch_barrier: AsyncArc::new(AsyncBarrier::new(2)),
            routes: RwLock::new(Vec::new()),
            status: AsyncArc::new(AsyncRwLock::new(HashMap::new())),
            headers: AsyncArc::new(AsyncRwLock::new(HashMap::new())),
            outgoing: AsyncArc::new(AsyncRwLock::new(HashMap::new())),
            shutdown: Stopper::new(),
        }
    }

    pub fn statuses(&self) -> AsyncArc<AsyncRwLock<HashMap<Uuid, AsyncProducerStatus>>> {
        AsyncArc::clone(&self.status)
    }

    pub fn headers(&self) -> AsyncArc<AsyncRwLock<HashMap<Uuid, AsyncProducerHeaders>>> {
        AsyncArc::clone(&self.headers)
    }

    pub fn outgoing(&self) -> AsyncArc<AsyncRwLock<HashMap<Uuid, AsyncProducerOutgoing>>> {
        AsyncArc::clone(&self.outgoing)
    }

    async fn continuous(&self) {
        let model = self.model.upgrade().unwrap();

        self.launch_barrier.wait().await;

        let routes = self.routes.read().unwrap().clone();

        let status = self.status.clone();
        let headers = self.headers.clone();
        let outgoing = self.outgoing.clone();

        let mut router = Router::new();
        for (method, route) in routes {
            let route = match RouteSpec::try_from(route.as_str()) {
                Ok(route) => route,
                Err(_) => continue,
            };

            let handler = {
                let route = Arc::new(route.clone());
                let status = Arc::clone(&status);
                let headers = Arc::clone(&headers);
                let outgoing = Arc::clone(&outgoing);
                let model = Arc::clone(&model);
                let method = Arc::clone(&method);

                move |mut conn: Conn| {
                    let route = Arc::clone(&route);
                    let status = Arc::clone(&status);
                    let headers = Arc::clone(&headers);
                    let outgoing = Arc::clone(&outgoing);
                    let model = Arc::clone(&model);
                    let method = Arc::clone(&method);

                    async move {
                        let id = Uuid::new_v4();
                        let http_request = HttpRequest {
                            id: id.as_u128(),
                            route: conn.route().map(|r| r.to_string()).unwrap_or_default(),
                            path: conn.path().to_string(),
                            parameters: Map::new_with(
                                route
                                    .segments()
                                    .iter()
                                    .filter_map(|seg| {
                                        if let Segment::Param(param) = seg {
                                            conn.param(param).map(|v| {
                                                (param.to_string(), Value::String(v.to_string()))
                                            })
                                        } else {
                                            None
                                        }
                                    })
                                    .collect(),
                            ),
                            method: (*method).clone(),
                        };

                        let params = {
                            let mut params = HashMap::new();
                            params.insert(
                                "method".to_string(),
                                Value::Data(Arc::clone(&method) as Arc<dyn Data>),
                            );
                            params.insert("route".to_string(), route.to_string().into());
                            params
                        };

                        let status_buf = AsyncHeapRb::<Status>::new(1);
                        let (status_prod, mut status_cons) = status_buf.split();
                        let headers_buf = AsyncHeapRb::<Map>::new(1);
                        let (headers_prod, mut headers_cons) = headers_buf.split();
                        let outgoing_buf = AsyncHeapRb::<u8>::new(2usize.pow(20));
                        let (prod, cons) = outgoing_buf.split();

                        status.write().await.insert(id, status_prod);
                        headers.write().await.insert(id, headers_prod);
                        outgoing.write().await.insert(id, prod);

                        let incoming_headers = conn
                            .request_headers()
                            .iter()
                            .filter_map(|(name, value)| {
                                value.as_str().map(|value| {
                                    (name.to_string(), Value::String(value.to_string()))
                                })
                            })
                            .collect();

                        // For now the reading of request is "one-shot", not allowing effective streaming of very large incoming requests.
                        let body = conn.request_body().await;
                        let (content, occured_failure) = match body.read_bytes().await {
                            Ok(content) => (content, None),
                            Err(err) => (Vec::new(), Some(err.to_string())),
                        };

                        model
                            .new_incoming(
                                None,
                                http_request,
                                &params,
                                Some(Box::new(move |mut outputs| {
                                    let started = outputs.get("started");
                                    let headers = outputs.get("headers");
                                    let data = outputs.get("data");
                                    let failure = outputs.get("failure");

                                    vec![Box::new(Box::pin(async move {
                                        if let Some(occured_failure) = occured_failure {
                                            let _ = failure.send_one(occured_failure.into()).await;
                                        } else {
                                            let _ = started.send_one(().into()).await;
                                            started.close().await;
                                            let _ = headers
                                                .send_one(Value::Data(Arc::new(Map::new_with(
                                                    incoming_headers,
                                                ))
                                                    as Arc<dyn Data>))
                                                .await;
                                            headers.close().await;
                                            let _ = data
                                                .send_many(TransmissionValue::Byte(content.into()))
                                                .await;
                                        }

                                        headers.close().await;
                                        data.close().await;
                                        failure.close().await;
                                        ResultStatus::Ok
                                    }))]
                                })),
                            )
                            .await;

                        if let (Some(status), Some(headers)) =
                            futures::join!(status_cons.pop(), headers_cons.pop())
                        {
                            conn.set_status(status);
                            conn.response_headers_mut()
                                .insert(KnownHeaderName::Server, SERVER);

                            for (name, content) in &headers.map {
                                let header_name = HeaderName::from(name.as_str());
                                if header_name.is_valid()
                                    && content.datatype().implements(
                                        &melodium_core::common::descriptor::DataTrait::ToString,
                                    )
                                {
                                    let header_content = HeaderValue::from(
                                        melodium_core::DataTrait::to_string(content),
                                    );
                                    if header_content.is_valid() {
                                        conn.response_headers_mut()
                                            .insert(header_name.to_owned(), header_content);
                                    }
                                }
                            }

                            conn.set_body(Body::new_streaming(cons, None));
                        } else {
                            conn.set_status(Status::InternalServerError);
                        }

                        conn.halt()
                    }
                }
            };

            match method.0 {
                Method::Delete => router = router.delete(route, handler),
                Method::Get => router = router.get(route, handler),
                Method::Patch => router = router.patch(route, handler),
                Method::Post => router = router.post(route, handler),
                Method::Put => router = router.put(route, handler),
                _ => {}
            }
        }

        trillium_async_std::config()
            .without_signals()
            .with_stopper(self.shutdown.clone())
            .with_port(model.get_port())
            .with_host(&model.get_host().0.to_string())
            .run_async(router)
            .await
    }

    fn invoke_source(&self, source: &str, params: HashMap<String, Value>) {
        match source {
            "incoming" => {
                let method: Arc<HttpMethod> = melodium_core::GetData::<Arc<dyn Data>>::try_data(
                    params.get("method").unwrap().clone(),
                )
                .unwrap()
                .downcast_arc()
                .unwrap();
                let route = melodium_core::GetData::<String>::try_data(
                    params.get("route").unwrap().clone(),
                )
                .unwrap();

                self.routes.write().unwrap().push((method, route));
            }
            _ => {}
        }
    }

    fn shutdown(&self) {
        self.shutdown.stop();
    }
}

#[mel_treatment(
    model http_server HttpServer
    input trigger Block<void>
)]
pub async fn start() {
    let model = HttpServerModel::into(http_server);
    let http_server = model.inner();

    if let Ok(_) = trigger.recv_one().await {
        http_server.launch_barrier.wait().await;
    }
}

#[mel_treatment(
    input status Block<HttpStatus>
    input headers Block<Map>
    input data Stream<byte>
    model http_server HttpServer
)]
pub async fn outgoing(id: u128) {
    let id = Uuid::from_u128(id);
    let model = HttpServerModel::into(http_server);
    let http_server = model.inner();

    let out_status;
    let out_headers;
    let output;
    {
        let statuses = http_server.statuses();
        let mut lock = statuses.write().await;
        out_status = lock.remove(&id);
    }
    {
        let headers = http_server.headers();
        let mut lock = headers.write().await;
        out_headers = lock.remove(&id);
    }
    {
        let outputs = http_server.outgoing();
        let mut lock = outputs.write().await;
        output = lock.remove(&id);
    }
    if let (Some(mut out_status), Some(mut out_headers), Some(mut output)) =
        (out_status, out_headers, output)
    {
        if let (Ok(status), Ok(headers)) = (
            status.recv_one().await.map(|val| {
                GetData::<Arc<dyn Data>>::try_data(val)
                    .unwrap()
                    .downcast_arc::<HttpStatus>()
                    .unwrap()
            }),
            headers.recv_one().await.map(|val| {
                GetData::<Arc<dyn Data>>::try_data(val)
                    .unwrap()
                    .downcast_arc::<Map>()
                    .unwrap()
            }),
        ) {
            match futures::join!(
                out_status.push(status.0),
                out_headers.push(Arc::unwrap_or_clone(headers))
            ) {
                (Ok(_), Ok(_)) => {
                    while let (Ok(data), false) = (
                        data.recv_many()
                            .await
                            .map(|values| TryInto::<Vec<byte>>::try_into(values).unwrap()),
                        output.is_closed(),
                    ) {
                        match output.push_iter(data.into_iter()).await {
                            Ok(_) => {}
                            Err(_) => break,
                        }
                    }
                }
                (_, _) => {}
            }
        }
    }
}
