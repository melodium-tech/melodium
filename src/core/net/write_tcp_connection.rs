
use crate::core::prelude::*;
use super::tcp_listener::TcpListenerModel;
use async_std::io::BufWriter;

treatment!(write_tcp_connection,
    core_identifier!("net";"WriteTcpConnection"),
    models![
        ("listener", super::super::tcp_listener::TcpListenerModel::descriptor())
    ],
    treatment_sources![
        (super::super::tcp_listener::TcpListenerModel::descriptor(), "connection")
    ],
    parameters![
        parameter!("ip", Scalar, String, None),
        parameter!("port", Scalar, U16, None)
    ],
    inputs![
        input!("data",Scalar, Byte, Stream)
    ],
    outputs![],
    host {
        use super::*;

        let listener = Arc::clone(&host.get_model("listener")).downcast_arc::<TcpListenerModel>().unwrap();
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
