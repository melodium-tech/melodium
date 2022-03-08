
use crate::core::prelude::*;

treatment!(file_writer_treatment,
    core_identifier!("fs","direct";"WriteFile"),
    models![
        ("writer", crate::core::fs::direct::file_writer::FileWriterModel::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("data",Scalar,Byte,Stream)
    ],
    outputs![],
    host {
        let writer = Arc::clone(&host.get_model("writer")).downcast_arc::<crate::core::fs::direct::file_writer::FileWriterModel>().unwrap();

        let input = host.get_input("data");
        let writer_sender = writer.writer().clone();
    
        while let Ok(bytes) = input.recv_byte().await {

            println!("Writing {} bytes", bytes.len());
            for byte in bytes {
                writer_sender.send(byte).await;
            }
        }
    
        ResultStatus::Ok
    }
);

