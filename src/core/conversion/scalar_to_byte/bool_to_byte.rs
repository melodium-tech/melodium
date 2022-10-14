
use crate::core::prelude::*;

treatment!(bool_to_byte,
    core_identifier!("conversion","scalar";"BoolToByte"),
    r"Convert stream of `bool` into `Vec<byte>`.

    Each `bool` gets converted into `Vec<byte>`, each vector contains the one byte of the former scalar `bool` it represents.".to_string(),
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
