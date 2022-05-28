
use crate::core::prelude::*;

treatment!(engine_ready_treatment,
    core_identifier!("engine";"Ready"),
    models![
        ("engine", crate::core::engine::engine::EngineModel::descriptor())
    ],
    treatment_sources![
        (crate::core::engine::engine::EngineModel::descriptor(), "ready")
    ],
    parameters![],
    inputs![
        input!("_ready",Scalar,Void,Block)
    ],
    outputs![
        output!("ready",Scalar,Void,Block)
    ],
    host {
        let input = host.get_input("_ready");
        let output = host.get_output("ready");
    
        if let Ok(_) = input.recv_one_void().await {

            let _ = output.send_void(()).await;
        }
    
        ResultStatus::Ok
    }
);
