use crate::method::*;
use crate::status::*;
use async_ringbuf::{AsyncHeapRb, AsyncProducer, AsyncRb};
use async_std::sync::{Arc as AsyncArc, RwLock as AsyncRwLock};
use core::{fmt::Debug, mem::MaybeUninit};
use melodium_core::{common::executive::ResultStatus, *};
use melodium_macro::{mel_context, mel_model, mel_treatment};
use ringbuf::SharedRb;
use routefinder::RouteSpec;
use std::sync::Arc;
use std::{
    collections::HashMap,
    sync::{RwLock, Weak},
};
use trillium::{Body, Conn};
use trillium_async_std::Stopper;
use trillium_router::{Router, RouterConnExt};
use uuid::Uuid;
use trillium::{Method, Status};

#[mel_context]
pub struct HttpRequest {
    pub id: u128,
    pub route: string,
    pub path: string,
}

type AsyncProducerStatus =
    AsyncProducer<Status, Arc<AsyncRb<Status, SharedRb<Status, Vec<MaybeUninit<Status>>>>>>;
type AsyncProducerOutgoing =
    AsyncProducer<u8, Arc<AsyncRb<u8, SharedRb<u8, Vec<MaybeUninit<u8>>>>>>;

#[mel_model(
    param host string none
    param port u16 none
    source incoming (HttpRequest) (
        param method HttpMethod none
        param route string none
    ) (
        data Stream<byte>
        failure Block<void>
    )
    source failed_binding () () (
        failure Block<string>
    )
    continuous (continuous)
    shutdown shutdown
)]
pub struct HttpServer {
    model: Weak<HttpServerModel>,
    //connections: Arc<RwLock<HashMap<u64, Sender<(u16, Vec<u8>)>>>>,
    routes: RwLock<Vec<(Arc<HttpMethod>, String)>>,
    status: AsyncArc<AsyncRwLock<HashMap<Uuid, AsyncProducerStatus>>>,
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
            routes: RwLock::new(Vec::new()),
            status: AsyncArc::new(AsyncRwLock::new(HashMap::new())),
            outgoing: AsyncArc::new(AsyncRwLock::new(HashMap::new())),
            shutdown: Stopper::new(),
        }
    }

    pub fn statuses(&self) -> AsyncArc<AsyncRwLock<HashMap<Uuid, AsyncProducerStatus>>> {
        AsyncArc::clone(&self.status)
    }

    pub fn outgoing(&self) -> AsyncArc<AsyncRwLock<HashMap<Uuid, AsyncProducerOutgoing>>> {
        AsyncArc::clone(&self.outgoing)
    }

    async fn continuous(&self) {
        let model = self.model.upgrade().unwrap();

        let routes = self.routes.read().unwrap().clone();

        let status = self.status.clone();
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
                let outgoing = Arc::clone(&outgoing);
                let model = Arc::clone(&model);
                let method = Arc::clone(&method);
                eprintln!("Preparing route handler {route:?}");
                move |mut conn: Conn| {
                    let route = Arc::clone(&route);
                    let status = Arc::clone(&status);
                    let outgoing = Arc::clone(&outgoing);
                    let model = Arc::clone(&model);
                    let method = Arc::clone(&method);

                    eprintln!("Preparing route {route:?}");

                    async move {
                        let id = Uuid::new_v4();
                        let http_request = HttpRequest {
                            id: id.as_u128(),
                            route: conn.route().map(|r| r.to_string()).unwrap_or_default(),
                            path: conn.path().to_string(),
                        };

                        eprintln!("Incoming request ({id})");

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
                        let outgoing_buf = AsyncHeapRb::<u8>::new(2usize.pow(20));
                        let (prod, cons) = outgoing_buf.split();

                        status.write().await.insert(id, status_prod);
                        outgoing.write().await.insert(id, prod);

                        let body = conn.request_body().await;
                        let (content, is_failure) = match body.read_bytes().await {
                            Ok(content) => (content, false),
                            Err(_) => (Vec::new(), true),
                        };

                        model
                            .new_incoming(
                                None,
                                http_request,
                                &params,
                                Some(Box::new(move |mut outputs| {
                                    let data = outputs.get("data");
                                    let failure = outputs.get("failure");

                                    vec![Box::new(Box::pin(async move {
                                        if is_failure {
                                            let _ = failure.send_one(().into()).await;
                                        } else {
                                            eprintln!("Received {} bytes", content.len());
                                            let _ = data
                                                .send_many(TransmissionValue::Byte(content.into()))
                                                .await;
                                        }

                                        data.close().await;
                                        failure.close().await;
                                        ResultStatus::Ok
                                    }))]
                                })),
                            )
                            .await;

                        if let Some(status) = status_cons.pop().await {
                            conn.set_status(status);
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

        eprintln!("{router:?}");

        trillium_async_std::config()
            .without_signals()
            .with_stopper(self.shutdown.clone())
            .with_port(model.get_port())
            .with_host(&model.get_host())
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
    input status Block<HttpStatus>
    input data Stream<byte>
    model http_server HttpServer
)]
pub async fn outgoing(id: u128) {
    let id = Uuid::from_u128(id);
    let model = HttpServerModel::into(http_server);
    let http_server = model.inner();

    let out_status;
    let output;
    {
        let statuses = http_server.statuses();
        let mut lock = statuses.write().await;
        out_status = lock.remove(&id);
    }
    {
        let outputs = http_server.outgoing();
        let mut lock = outputs.write().await;
        output = lock.remove(&id);
    }
    if let (Some(mut out_status), Some(mut output)) = (out_status, output) {
        if let Ok(status) = status.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<HttpStatus>()
                .unwrap()
        }) {
            match out_status.push(status.0).await {
                Ok(_) => {
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
                Err(_) => {}
            }
        }
    }
}
