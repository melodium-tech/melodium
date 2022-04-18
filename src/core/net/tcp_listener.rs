
use super::super::prelude::*;
use async_std::net::*;

#[derive(Debug)]
pub struct TcpListenerModel {

    helper: ModelHelper,

    available_streams: RwLock<HashMap<(String, u16), TcpStream>>,

    auto_reference: Weak<Self>,
}

impl TcpListenerModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {

        model_desc!(
            TcpListenerModel,
            core_identifier!("net";"TcpListener"),
            vec![
                parameter!("socket_address", Scalar, String, None),
            ],
            model_sources![
                ("connection"; "TcpConnection")
            ]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        Arc::new_cyclic(|me| Self {
            helper: ModelHelper::new(Self::descriptor(), world),

            available_streams: RwLock::new(HashMap::new()),

            auto_reference: me.clone(),
        })
    }

    pub fn available_streams(&self) -> &RwLock<HashMap<(String, u16), TcpStream>> {
        &self.available_streams
    }

    fn initialize(&self) {

        let auto_self = self.auto_reference.upgrade().unwrap();
        let continuous_future = Box::pin(async move { auto_self.listen().await });

        self.helper.world().add_continuous_task(Box::new(continuous_future));
    }

    async fn listen(&self) {

        let socket_address: SocketAddr = self.helper.get_parameter("socket_address").string().parse().unwrap();

        // Todo manage io error
        if let Ok(listener) = TcpListener::bind(socket_address).await {

            let local_socket = listener.local_addr().unwrap();

            while let Ok((stream, addr)) = listener.accept().await {

                self.available_streams.write().unwrap().insert((addr.ip().to_string(), addr.port()), stream.clone());

                let data_reading = |inputs| {
                    self.stream_read(stream.clone(), inputs)
                };

                let mut tcp_connection_context = Context::new();

                tcp_connection_context.set_value("localIp", Value::String(local_socket.ip().to_string()));
                tcp_connection_context.set_value("localPort", Value::U16(local_socket.port()));
                tcp_connection_context.set_value("remoteIp", Value::String(addr.ip().to_string()));
                tcp_connection_context.set_value("remotePort", Value::U16(addr.port()));
                tcp_connection_context.set_value("isIpV4", Value::Bool(addr.is_ipv4()));
                tcp_connection_context.set_value("isIpV6", Value::Bool(addr.is_ipv6()));

                let mut contextes = HashMap::new();
                contextes.insert("TcpConnection".to_string(), tcp_connection_context);

                let model_id = self.helper.id().unwrap();
                self.helper.world().create_track(model_id, "connection", contextes, None, Some(data_reading)).await;
            }
        }

        // Todo manage failures
    }

    fn stream_read(&self, mut stream: TcpStream, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let data_output = inputs.get("_data").unwrap().clone();

        let future = Box::new(Box::pin(async move {

            let mut buf = vec![0u8; 1024];
            while let Ok(num) = stream.read(&mut buf).await {

                // Tcp-specific behavior
                if num == 0 {
                    break;
                }

                ok_or_break!(data_output.send_multiple_byte(buf.clone()).await);
            }

            data_output.close().await;

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }
}

model_trait!(TcpListenerModel, initialize);
