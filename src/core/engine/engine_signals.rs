
use crate::core::prelude::*;

treatment!(engine_sighup_treatment,
    core_identifier!("engine";"Sighup"),
    models![
        ("engine", crate::core::engine::engine::EngineModel::descriptor())
    ],
    treatment_sources![
        (crate::core::engine::engine::EngineModel::descriptor(), "sighup")
    ],
    parameters![],
    inputs![
        input!("_sighup",Scalar,Void,Block)
    ],
    outputs![
        output!("sighup",Scalar,Void,Block)
    ],
    host {
        let input = host.get_input("_sighup");
        let output = host.get_output("sighup");
    
        if let Ok(_) = input.recv_one_void().await {

            let _ = output.send_void(()).await;
        }
    
        ResultStatus::Ok
    }
);

treatment!(engine_sigterm_treatment,
    core_identifier!("engine";"Sigterm"),
    models![
        ("engine", crate::core::engine::engine::EngineModel::descriptor())
    ],
    treatment_sources![
        (crate::core::engine::engine::EngineModel::descriptor(), "sigterm")
    ],
    parameters![],
    inputs![
        input!("_sigterm",Scalar,Void,Block)
    ],
    outputs![
        output!("sigterm",Scalar,Void,Block)
    ],
    host {
        let input = host.get_input("_sigterm");
        let output = host.get_output("sigterm");
    
        if let Ok(_) = input.recv_one_void().await {

            let _ = output.send_void(()).await;
        }
    
        ResultStatus::Ok
    }
);
