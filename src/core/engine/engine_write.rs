
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

        // We do keep the writer model because might be good in further improvement to signal when stdout is closed.
        let _writer = Arc::clone(&host.get_model("engine")).downcast_arc::<crate::core::engine::engine::EngineModel>().unwrap();

        let input = host.get_input("text");
    
        'main: while let Ok(text) = input.recv_string().await {

            let mut stdout = async_std::io::stdout();

            for part in text {
                ok_or_break!('main, stdout.write_all(part.as_bytes()).await);
            }

            ok_or_break!(stdout.flush().await);
        }

        ResultStatus::Ok
    }
);
