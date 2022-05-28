
use crate::core::prelude::*;

treatment!(engine_read_treatment,
    core_identifier!("engine";"Read"),
    models![
        ("engine", crate::core::engine::engine::EngineModel::descriptor())
    ],
    treatment_sources![
        (crate::core::engine::engine::EngineModel::descriptor(), "read")
    ],
    parameters![],
    inputs![
        input!("_line",Scalar,String,Stream)
    ],
    outputs![
        output!("line",Scalar,String,Stream)
    ],
    host {
        let input = host.get_input("_line");
        let output = host.get_output("line");
    
        while let Ok(lines) = input.recv_string().await {

            ok_or_break!(output.send_multiple_string(lines).await);
        }
    
        ResultStatus::Ok
    }
);
