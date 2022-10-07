
use crate::core::prelude::*;

treatment!(read_file_treatment,
    core_identifier!("fs","read";"ReadFile"),
    models![
        ("reader", crate::core::fs::read::files_reader::FilesReaderModel::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("path",Scalar,String,Stream)
    ],
    outputs![],
    host {

        let reader = std::sync::Arc::clone(&host.get_model("reader")).downcast_arc::<crate::core::fs::read::files_reader::FilesReaderModel>().unwrap();

        let path_input = host.get_input("path");

        while let Ok(paths) = path_input.recv_string().await {

            for path in paths {
                reader.read(&path).await;
            }
        }
    
        ResultStatus::Ok
    }
);
