
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
        let writer_sender = SendTransmitter::new();
        writer_sender.add_transmitter(writer.writer());
    
        while let Ok(bytes) = input.recv_byte().await {

            ok_or_break!(writer_sender.send_multiple(bytes).await);
        }

        writer_sender.close().await;
    
        ResultStatus::Ok
    }
);

