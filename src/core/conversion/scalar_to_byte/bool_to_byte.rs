
use crate::core::prelude::*;

treatment!(bool_to_byte,
    core_identifier!("conversion","scalar";"BoolToByte"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("value",Scalar,Bool,Stream)
    ],
    outputs![
        output!("data",Vector,Byte,Stream)
    ],
    host {
        let input = host.get_input("value");
        let output = host.get_output("data");
    
        'main: while let Ok(bools) = input.recv_bool().await {

            ok_or_break!('main, output.send_multiple_vec_byte(bools.iter().map(
                |b|
                vec![match b {
                    true => 1,
                    false => 0,
                }]
            ).collect()).await);
        }
    
        ResultStatus::Ok
    }
);
