
use crate::core::prelude::*;

treatment!(file_reader_treatment,
    core_identifier!("fs","direct";"ReadFile"),
    models![
        ("reader", crate::core::fs::read::files_reader::FileReaderModel::descriptor())
    ],
    treatment_sources![
        (crate::core::fs::direct::file_reader::FileReaderModel::descriptor(), "read")
    ],
    parameters![],
    inputs![
        input!("_data",Scalar,Byte,Stream),
        input!("_failure",Scalar,Void,Block),
        input!("_message",Scalar,String,Stream)
    ],
    outputs![
        output!("data",Scalar,Byte,Stream),
        output!("failure",Scalar,Void,Block),
        output!("message",Scalar,String,Stream)
    ],
    host {
        let i_data = host.get_input("_data");
        let o_data = host.get_output("data");

        let i_failure = host.get_input("_failure");
        let o_failure = host.get_output("failure");

        let i_message = host.get_input("_message");
        let o_message = host.get_output("message");
    
        while let Ok(bytes) = i_data.recv_byte().await {

            ok_or_break!(o_data.send_multiple_byte(bytes).await);
        }

        if let (Ok(failure), Ok(messages)) = futures::join!(i_failure.recv_one_void(), i_message.recv_string()) {

            let _ = o_failure.send_void(()).await;
            let _ = o_message.send_multiple_string(messages).await;
        }
    
        ResultStatus::Ok
    }
);
