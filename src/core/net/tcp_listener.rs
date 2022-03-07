
use super::super::prelude::*;
use async_std::net::*;

#[derive(Debug)]
pub struct TcpListenerModel {

    world: Arc<World>,
    id: RwLock<Option<ModelId>>,

    socket_address: RwLock<String>,

    available_streams: RwLock<HashMap<(String, u16), TcpStream>>,

    auto_reference: RwLock<Weak<Self>>,
}

impl TcpListenerModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreModelDescriptor> = {

                let builder = CoreModelBuilder::new(TcpListenerModel::new);

                let descriptor = CoreModelDescriptor::new(
                    core_identifier!("net";"TcpListener"),
                    vec![
                        parameter!("socket_address", Scalar, String, None)
                    ],
                    model_sources![
                        ("connection"; "TcpConnection")
                    ],
                    Box::new(builder)
                );

                let rc_descriptor = Arc::new(descriptor);
                rc_descriptor.set_autoref(&rc_descriptor);

                rc_descriptor
            };
        }
        
        Arc::clone(&DESCRIPTOR)
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        let model = Arc::new(Self {
            world,
            id: RwLock::new(None),

            socket_address: RwLock::new(String::new()),

            available_streams: RwLock::new(HashMap::new()),

            auto_reference: RwLock::new(Weak::new()),
        });

        *model.auto_reference.write().unwrap() = Arc::downgrade(&model);

        model
    }

    pub fn socket_address(&self) -> String {
        self.socket_address.read().unwrap().clone()
    }

    pub fn available_streams(&self) -> &RwLock<HashMap<(String, u16), TcpStream>> {
        &self.available_streams
    }

    async fn listen(&self) {

        let socket_address: SocketAddr = self.socket_address().parse().unwrap();

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

                let model_id = self.id.read().unwrap().unwrap();
                let inputs = self.world.create_track(model_id, "connection", contextes, None, Some(data_reading)).await;
            }
        }

        // Todo manage failures
    }

    fn stream_read(&self, mut stream: TcpStream, inputs: HashMap<String, Vec<Input>>) -> Vec<TrackFuture> {

        let data_output_transmitters = inputs.get("_data").unwrap().clone();

        let data_output = Output::Byte(Arc::new(SendTransmitter::new()));
        inputs.get("_data").unwrap().iter().for_each(|i| data_output.add_input(i));

        let future = Box::new(Box::pin(async move {

            let mut buf = vec![0u8; 1024];
            while let Ok(num) = stream.read(&mut buf).await {

                // Tcp-specific behavior
                if num == 0 {
                    break;
                }

                data_output.send_multiple_byte(buf.clone()).await;
            }

            data_output.close();

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }
}

impl Model for TcpListenerModel {
    
    fn descriptor(&self) -> Arc<CoreModelDescriptor> {
        Self::descriptor()
    }

    fn id(&self) -> Option<ModelId> {
        *self.id.read().unwrap()
    }

    fn set_id(&self, id: ModelId) {
        *self.id.write().unwrap() = Some(id);
    }

    fn set_parameter(&self, param: &str, value: &Value) {

        match param {
            "socket_address" => {
                match value {
                    Value::String(path) => *self.socket_address.write().unwrap() = path.to_string(),
                    _ => panic!("Unexpected value type for 'socket_address'."),
                }
            },
            _ => panic!("No parameter '{}' exists.", param)
        }
    }

    fn get_context_for(&self, source: &str) -> Vec<String> {

        Vec::new()
    }

    fn initialize(&self) {

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let continuous_future = Box::pin(async move { auto_self.listen().await });

        self.world.add_continuous_task(Box::new(continuous_future));
    }

    fn shutdown(&self) {

    }
}
