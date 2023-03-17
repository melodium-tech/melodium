
use crate::core::prelude::*;

treatment!(bool_from_byte,
    core_identifier!("conversion","scalar";"BoolFromByte"),
    indoc!(r"Convert stream of `Vec<byte>` into `bool`.

    Each received `byte` vector try to be converted into `bool`, and if valid is sent as `value`. If the incoming vector 
    is not valid for representing a `bool` (i.e. not having size of 1) it is refused and sent through `reject`.").to_string(),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("data",Vector,Byte,Stream)
    ],
    outputs![
        output!("value",Scalar,Bool,Stream),
        output!("reject",Vector,Byte,Stream)
    ],
    host {
        let input = host.get_input("data");
        let accept = host.get_output("value");
        let reject = host.get_output("reject");

        let mut accepted_op = true;
        let mut rejected_op = true;
    
        'main: while let Ok(vectors) = input.recv_vec_byte().await {
    
            for v in vectors {
                if v.len() == 1 {

                    let b = if v[0] == 0 { false } else { true };

                    if let Err(_) = accept.send_bool(b).await {
                        // If we cannot send anymore on accepted, we note it,
                        // and check if rejected is still valid, else just terminate.
                        accepted_op = false;
                        if !rejected_op {
                            break 'main;
                        }
                    }
                }
                else {
                    if let Err(_) = reject.send_vec_byte(v).await {
                        // If we cannot send anymore on rejected, we note it,
                        // and check if accepted is still valid, else just terminate.
                        rejected_op = false;
                        if !accepted_op {
                            break 'main;
                        }
                    }
                }
            }
        }
    
        ResultStatus::Ok
    }
);
