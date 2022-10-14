
use crate::core::prelude::*;

treatment!(char_from_byte,
    core_identifier!("conversion","scalar";"CharFromByte"),
    r"Convert stream of `Vec<byte>` into `char`.

    Each received `byte` vector try to be converted into `char`, and if valid is sent as `value`. If the incoming vector 
    is not valid for representing a `char` (i.e. not containing a valid [Unicode scalar value](https://www.unicode.org/glossary/#unicode_scalar_value)) it is refused and sent through `reject`.".to_string(),
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
