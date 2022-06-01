
use crate::core::prelude::*;

treatment!(engine_end_treatment,
    core_identifier!("engine";"End"),
    models![
        ("engine", crate::core::engine::engine::EngineModel::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("end",Scalar,Void,Block)
    ],
    outputs![],
    host {

        let engine = Arc::clone(&host.get_model("engine")).downcast_arc::<crate::core::engine::engine::EngineModel>().unwrap();

        let input = host.get_input("end");
    
        if let Ok(_) = input.recv_one_void().await {

            engine.end();
        }
    
        ResultStatus::Ok
    }
);
