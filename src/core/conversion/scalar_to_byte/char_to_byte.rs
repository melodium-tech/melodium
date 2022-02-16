
use crate::core::prelude::*;

treatment!(char_to_byte,
    core_identifier!("conversion","scalar";"CharToByte"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("value",Scalar,Char,Stream)
    ],
    outputs![
        output!("data",Scalar,Byte,Stream)
    ],
    host {
        let input = host.get_input("value");
        let output = host.get_output("data");
    
        while let Ok(chars) = input.recv_char().await {
    
            for ch in chars {
                output.send_multiple_byte(ch.to_string().as_bytes().to_vec()).await;
            }
        }
    
        ResultStatus::Ok
    }
);
