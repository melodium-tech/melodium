
use std::sync::{Arc, Weak, RwLock};
use std::collections::HashMap;
use super::super::prelude::*;
use async_std::net::*;
use async_std::io::BufWriter;

#[derive(Debug)]
pub struct TcpListenerModel {

    host: Weak<ModelHost>,

    available_streams: RwLock<HashMap<(String, u16), TcpStream>>,

    auto_reference: Weak<Self>,
}

impl TcpListenerModel {

    pub fn new(host: Weak<ModelHost>) -> Arc<dyn HostedModel> {

        Arc::new_cyclic(|me| Self {
            host,

            available_streams: RwLock::new(HashMap::new()),

            auto_reference: me.clone(),
        })
    }

    pub fn available_streams(&self) -> &RwLock<HashMap<(String, u16), TcpStream>> {
        &self.available_streams
    }

    async fn listen(&self) {

        let host = self.host.upgrade().unwrap();

        let socket_address: SocketAddr = host.get_parameter("socket_address").string().parse().unwrap();

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

                let model_id = host.id().unwrap();
                host.world().create_track(model_id, "connection", contextes, None, Some(data_reading)).await;
            }
        }

        // Todo manage failures
    }

    fn stream_read(&self, mut stream: TcpStream, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let data_output = inputs.get("data").unwrap().clone();

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

impl HostedModel for TcpListenerModel {

    fn initialize(&self) {
        let auto_self = self.auto_reference.upgrade().unwrap();
        let continuous_future = Box::pin(async move { auto_self.listen().await });

        self.host.upgrade().unwrap().world().add_continuous_task(Box::new(continuous_future));
    }

    fn shutdown(&self) {}
}

model!(
    TcpListenerModel,
    core_identifier!("net";"TcpListener"),
    parameters![
        parameter!("socket_address", Scalar, String, None)
    ],
    model_sources![
        ("connection"; "TcpConnection")
    ]
);

source!(read_tcp_connection,
    core_identifier!("net";"ReadTcpConnection"),
    models![
        ("listener", super::super::tcp_listener::model_host::descriptor())
    ],
    treatment_sources![
        (super::super::tcp_listener::model_host::descriptor(), "connection")
    ],
    outputs![
        output!("data", Scalar, Byte, Stream)
    ]
);

treatment!(write_tcp_connection,
    core_identifier!("net";"WriteTcpConnection"),
    models![
        ("listener", super::super::tcp_listener::model_host::descriptor())
    ],
    treatment_sources![
        (super::super::tcp_listener::model_host::descriptor(), "connection")
    ],
    parameters![
        parameter!("ip", Var, Scalar, String, None),
        parameter!("port", Var, Scalar, U16, None)
    ],
    inputs![
        input!("data", Scalar, Byte, Stream)
    ],
    outputs![],
    host {
        use super::*;

        let listener = host.get_hosted_model("listener").downcast_arc::<TcpListenerModel>().unwrap();
        let ip = host.get_parameter("ip").string();
        let port = host.get_parameter("port").u16();

        let input = host.get_input("data");

        let stream = listener.available_streams().read().unwrap().get(&(ip.to_string(), port)).unwrap().clone();
        let mut writer = BufWriter::with_capacity(1024, stream);
    
        while let Ok(bytes) = input.recv_byte().await {

            if let Err(write_err) = writer.write(&bytes).await {
                // Todo handle error
                panic!("Writing error: {}", write_err)
            }
        }

        if let Err(write_err) = writer.flush().await {

            // Todo handle error
            panic!("Writing (flush) error: {}", write_err)
        }
    
        ResultStatus::Ok
    }
);
