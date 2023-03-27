
use std::sync::{Arc, Weak};
use std::collections::HashMap;
use crate::core::prelude::*;
use async_std::path::PathBuf;
use async_std::fs::File;

#[derive(Debug)]
pub struct FilesReaderModel {

    host: Weak<ModelHost>,
}

impl FilesReaderModel {

    pub fn new(host: Weak<ModelHost>) -> Arc<dyn HostedModel> {

        Arc::new(Self {
            host
        })
    }

    pub async fn read(&self, path: &String) {

        let host = self.host.upgrade().unwrap();

        let model_id = host.id().unwrap();

        let os_path = PathBuf::from(path);
        let open_result = File::open(&os_path).await;

        let mut file_context = Context::new();

        let path = if let Ok(os_string) = os_path.canonicalize().await {
            os_string.into_os_string().into_string().unwrap_or_default()
        } else { "".to_string() };
        file_context.set_value("path", Value::String(path));

        let directory = if let Some(path) = os_path.parent() {
            if let Some(path) = path.to_str() {
                path.to_string()
            }
            else { "".to_string() }
        }
        else { "".to_string() };
        file_context.set_value("directory", Value::String(directory));

        let name = if let Some(name) = os_path.file_name() {
            if let Some(name) = name.to_str() {
                name.to_string()
            }
            else { "".to_string() }
        }
        else { "".to_string() };
        file_context.set_value("name", Value::String(name));

        let stem = if let Some(stem) = os_path.file_stem() {
            if let Some(stem) = stem.to_str() {
                stem.to_string()
            }
            else { "".to_string() }
        }
        else { "".to_string() };
        file_context.set_value("stem", Value::String(stem));

        let extension = if let Some(extension) = os_path.file_stem() {
            if let Some(extension) = extension.to_str() {
                extension.to_string()
            }
            else { "".to_string() }
        }
        else { "".to_string() };
        file_context.set_value("extension", Value::String(extension));

        let mut contextes = HashMap::new();
        contextes.insert("File".to_string(), file_context);

        match open_result {
            Ok(file) => {

                let reader = |inputs| {
                    self.read_file(file, inputs)
                };
                host.world().create_track(model_id, "read", contextes, None, Some(reader)).await;
            },
            Err(err) => {

                let failer = |inputs| {
                    self.fail_file(err, inputs)
                };
                host.world().create_track(model_id, "unaccessible", contextes, None, Some(failer)).await;
            },
        }
    }

    fn read_file(&self, mut file: File, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            let data_output = inputs.get("data").unwrap();
            let failure_output = inputs.get("failure").unwrap();
            let message_output = inputs.get("message").unwrap();

            let mut buf = vec![0; 1048576];
            loop {

                match file.read(&mut buf).await {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }

                        ok_or_break!(data_output.send_multiple_byte(buf.get(0..n).unwrap().to_vec()).await);
                    },
                    Err(err) => {

                        let _ = futures::join!(failure_output.send_void(()),message_output.send_string(format!("{:?}", err.kind())));
                    },
                }
            }

            futures::join!(data_output.close(), failure_output.close(), message_output.close());

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }

    fn fail_file(&self, err: async_std::io::Error, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            let failure_output = inputs.get("failure").unwrap();
            let message_output = inputs.get("message").unwrap();

            let _ = futures::join!(failure_output.send_void(()),message_output.send_string(format!("{:?}", err.kind())));

            futures::join!(failure_output.close(), message_output.close());

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }
}

impl HostedModel for FilesReaderModel {

    fn initialize(&self) {}
    fn shutdown(&self) {}
}

model!(
    FilesReaderModel,
    core_identifier!("fs","read";"FilesReader"),
    "Files reader model".to_string(),
    parameters![],
    model_sources![
        ("read"; "File"),
        ("unaccessible"; "File")
    ]
);

source!(reading_source,
    core_identifier!("fs","read";"Reading"),
    indoc!(r#"Process file reading
    
    The contents of the files given through `path` are streamed through `data`.
    A new track is created for each received path.

    If any reading failure happens, `failure` is emitted and `message` contains text of the related error."#).to_string(),
    models![
        ("reader", crate::core::fs::read::files_reader::model_host::descriptor())
    ],
    treatment_sources![
        (crate::core::fs::read::files_reader::model_host::descriptor(), "read")
    ],
    outputs![
        output!("data",Scalar,Byte,Stream),
        output!("failure",Scalar,Void,Block),
        output!("message",Scalar,String,Stream)
    ]
);

source!(unaccessible_source,
    core_identifier!("fs","read";"Unaccessible"),
    indoc!(r#"Process unaccessible file
    
    `failure` is emitted and `message` contains text of the related error."#).to_string(),
    models![
        ("reader", crate::core::fs::read::files_reader::model_host::descriptor())
    ],
    treatment_sources![
        (crate::core::fs::read::files_reader::model_host::descriptor(), "unaccessible")
    ],
    outputs![
        output!("failure",Scalar,Void,Block),
        output!("message",Scalar,String,Stream)
    ]
);

treatment!(read_file_treatment,
    core_identifier!("fs","read";"ReadFile"),
    indoc!(r#"Trigger reading of files
    
    Each `path` given is processed for reading."#).to_string(),
    models![
        ("reader", crate::core::fs::read::files_reader::model_host::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("path",Scalar,String,Stream)
    ],
    outputs![],
    host {

        let reader = host.get_hosted_model("reader").downcast_arc::<crate::core::fs::read::files_reader::FilesReaderModel>().unwrap();

        let path_input = host.get_input("path");

        while let Ok(paths) = path_input.recv_string().await {

            for path in paths {
                reader.read(&path).await;
            }
        }
    
        ResultStatus::Ok
    }
);

