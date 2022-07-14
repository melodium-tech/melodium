
use crate::core::prelude::*;

treatment!(file_reader_treatment,
    core_identifier!("fs","read";"ReadFile"),
    models![
        ("reader", crate::core::fs::read::files_reader::FileReaderModel::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("path",Scalar,String,Stream)
    ],
    outputs![],
    host {

        let reader = Arc::clone(&host.get_model("reader")).downcast_arc::<crate::core::fs::read::files_reader::FileReaderModel>().unwrap();

        let path_input = host.get_input("path");

        while let Ok(paths) = path_input.recv_string().await {

            for path in paths {
                reader.read(&path).await;
            }
        }
    
        ResultStatus::Ok
    }
);
