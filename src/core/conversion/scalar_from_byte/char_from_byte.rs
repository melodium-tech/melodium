
use crate::core::prelude::*;

treatment!(char_from_byte,
    core_identifier!("conversion","scalar";"CharFromByte"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("data",Vector,Byte,Stream)
    ],
    outputs![
        output!("value",Scalar,Char,Stream),
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
                if v.len() == 4 {

                    let code = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);

                    if let Some(c) = char::from_u32(code) {
                        if let Err(_) = accept.send_char(c).await {
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
