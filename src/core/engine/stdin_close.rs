
use crate::core::prelude::*;

treatment!(stdin_close_treatment,
    core_identifier!("engine";"Close"),
    models![
        ("stdin", crate::core::engine::stdin::StdinModel::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("close",Scalar,Void,Block)
    ],
    outputs![],
    host {

        let stdin = Arc::clone(&host.get_model("stdin")).downcast_arc::<crate::core::engine::stdin::StdinModel>().unwrap();

        let input = host.get_input("close");

        if let Ok(_) = input.recv_void().await {
            stdin.close();
        }
    
        ResultStatus::Ok
    }
);
