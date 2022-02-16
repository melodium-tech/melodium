
use crate::core::prelude::*;

treatment!(string_to_byte,
    core_identifier!("conversion","scalar";"StringToByte"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("value",Scalar,String,Stream)
    ],
    outputs![
        output!("data",Scalar,Byte,Stream)
    ],
    host {
        let input = host.get_input("value");
        let output = host.get_output("data");
    
        while let Ok(strings) = input.recv_string().await {
    
            for string in strings {
                output.send_multiple_byte(string.as_bytes().to_vec()).await;
            }
        }
    
        ResultStatus::Ok
    }
);
