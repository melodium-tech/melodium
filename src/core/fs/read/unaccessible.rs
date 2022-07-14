
use crate::core::prelude::*;

treatment!(unaccessible_treatment,
    core_identifier!("fs","read";"Unaccessible"),
    models![
        ("reader", crate::core::fs::read::files_reader::FilesReaderModel::descriptor())
    ],
    treatment_sources![
        (crate::core::fs::read::files_reader::FilesReaderModel::descriptor(), "unaccessible")
    ],
    parameters![],
    inputs![
        input!("_failure",Scalar,Void,Block),
        input!("_message",Scalar,String,Stream)
    ],
    outputs![
        output!("failure",Scalar,Void,Block),
        output!("message",Scalar,String,Stream)
    ],
    host {
        let i_failure = host.get_input("_failure");
        let o_failure = host.get_output("failure");

        let i_message = host.get_input("_message");
        let o_message = host.get_output("message");

        if let (Ok(_), Ok(messages)) = futures::join!(i_failure.recv_one_void(), i_message.recv_string()) {

            let _ = o_failure.send_void(()).await;
            let _ = o_message.send_multiple_string(messages).await;
        }
    
        ResultStatus::Ok
    }
);
