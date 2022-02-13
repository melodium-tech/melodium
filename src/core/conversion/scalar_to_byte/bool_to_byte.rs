
use crate::core::prelude::*;

treatment!(bool_to_byte,
    core_identifier!("conversion";"BoolToByte"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("value",Scalar,Bool,Stream)
    ],
    outputs![
        output!("data",Scalar,Byte,Stream)
    ],
    host {
        let input = host.get_input("value");
        let output = host.get_output("data");
    
        while let Ok(bools) = input.recv_bool().await {

            output.send_multiple_byte(bools.iter().map(
                |b|
                match b {
                    true => 1,
                    false => 0,
                }
            ).collect()).await;
        }
    
        ResultStatus::Ok
    }
);
