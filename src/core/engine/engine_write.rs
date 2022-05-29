
use crate::core::prelude::*;

treatment!(engine_write_treatment,
    core_identifier!("engine";"Write"),
    models![
        ("engine", crate::core::engine::engine::EngineModel::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("text",Scalar,String,Stream)
    ],
    outputs![],
    host {

        let writer = Arc::clone(&host.get_model("engine")).downcast_arc::<crate::core::engine::engine::EngineModel>().unwrap();

        let input = host.get_input("text");
        let writer_sender = SendTransmitter::new();
        writer_sender.add_transmitter(writer.writer());
    
        while let Ok(text) = input.recv_string().await {

            // TODO enable this once engine have end trigger
            //ok_or_break!(writer_sender.send_multiple(text).await);
            // and delete this
            for t in text {
                print!("{}", t);
            }
        }

        writer_sender.close().await;
    
        ResultStatus::Ok
    }
);
