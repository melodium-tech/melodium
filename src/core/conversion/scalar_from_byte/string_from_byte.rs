
use crate::core::prelude::*;

treatment!(string_from_byte,
    core_identifier!("conversion","scalar";"StringFromByte"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("data",Vector,Byte,Stream)
    ],
    outputs![
        output!("value",Scalar,String,Stream),
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
                if let Ok(s) = String::from_utf8(v.clone()) {
                    if let Err(_) = accept.send_string(s).await {
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
