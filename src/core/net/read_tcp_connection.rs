
use crate::core::prelude::*;

treatment!(read_tcp_connection,
    core_identifier!("net";"ReadTcpConnection"),
    models![
        ("listener", super::super::tcp_listener::TcpListenerModel::descriptor())
    ],
    treatment_sources![
        (super::super::tcp_listener::TcpListenerModel::descriptor(), "connection")
    ],
    parameters![],
    inputs![
        input!("_data",Scalar, Byte, Stream)
    ],
    outputs![
        output!("data", Scalar, Byte, Stream)
    ],
    host {
        let input = host.get_input("_data");
        let output = host.get_output("data");
    
        while let Ok(bytes) = input.recv_byte().await {

            ok_or_break!(output.send_multiple_byte(bytes).await);
        }
    
        ResultStatus::Ok
    }
);
