
use crate::core::prelude::*;

treatment!(file_reader_treatment,
    core_identifier!("fs","direct";"ReadFile"),
    models![
        ("reader", crate::core::fs::direct::file_reader::FileReaderModel::descriptor())
    ],
    treatment_sources![
        (crate::core::fs::direct::file_reader::FileReaderModel::descriptor(), "read")
    ],
    parameters![],
    inputs![
        input!("_data",Scalar,Byte,Stream)
    ],
    outputs![
        output!("data",Scalar,Byte,Stream)
    ],
    host {
        let input = host.get_input("_data");
        let output = host.get_output("data");
    
        while let Ok(bytes) = input.recv_byte().await {

            output.send_multiple_byte(bytes).await;
        }
    
        ResultStatus::Ok
    }
);


