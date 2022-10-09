
use std::sync::{Arc, Weak};
use crate::core::prelude::*;

#[derive(Debug)]
pub struct FilesWriterModel {

    host: Weak<ModelHost>,
}

impl FilesWriterModel {

    pub fn new(host: Weak<ModelHost>) -> Arc<dyn HostedModel> {

        Arc::new(Self {
            host
        })
    }

    pub fn append(&self) -> bool {
        self.host.upgrade().unwrap().get_parameter("append").bool()
    }

    pub fn create(&self) -> bool {
        self.host.upgrade().unwrap().get_parameter("create").bool()
    }

    pub fn create_new(&self) -> bool {
        self.host.upgrade().unwrap().get_parameter("new").bool()
    }
}

impl HostedModel for FilesWriterModel {

    fn initialize(&self) {}
    fn shutdown(&self) {}
}

model!(
    FilesWriterModel,
    core_identifier!("fs","write";"FilesWriter"),
    parameters![
        parameter!("append", Scalar, Bool, Some(Value::Bool(false))),
        parameter!("create", Scalar, Bool, Some(Value::Bool(true))),
        parameter!("new", Scalar, Bool, Some(Value::Bool(false)))
    ],
    model_sources![]
);


treatment!(write_file_treatment,
    core_identifier!("fs","write";"WriteFile"),
    models![
        ("writer", crate::core::fs::write::files_writer::model_host::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("path",Scalar,String,Block),
        input!("data",Scalar,Byte,Stream)
    ],
    outputs![
        output!("amount",Scalar,U64,Stream),
        output!("success",Scalar,Void,Block),
        output!("failure",Scalar,Void,Block),
        output!("message",Scalar,String,Stream)
    ],
    host {

        let writer = host.get_hosted_model("writer").downcast_arc::<crate::core::fs::write::files_writer::FilesWriterModel>().unwrap();

        let i_path = host.get_input("path");
        let i_data = host.get_input("data");

        let o_amount = host.get_output("amount");
        let o_success = host.get_output("success");
        let o_failure = host.get_output("failure");
        let o_message = host.get_output("message");

        if let Ok(path) = i_path.recv_one_string().await {

            let os_path = async_std::path::PathBuf::from(path);

            let mut open_options = async_std::fs::OpenOptions::new();
            open_options
                .write(true)
                .append(writer.append())
                .create(writer.create())
                .create_new(writer.create_new());

            let open_result = open_options.open(&os_path).await;

            match open_result {
                Ok(mut file) => {
    
                    let mut error = false;
                    // We don't handle the error case as it means everything is empty and closed
                    while let Ok(data) = i_data.recv_byte().await {
                        if let Err(err) = file.write_all(&data).await {
        
                            let _ = futures::join!(o_failure.send_void(()),o_message.send_string(format!("{:?}", err.kind())));
                            error = true;
                            break;
                        }
                        else {
                            let _ = o_amount.send_u64(data.len() as u64);
                        }
                    }

                    if !error {
                        if let Err(err) = file.flush().await {
        
                            let _ = futures::join!(o_failure.send_void(()),o_message.send_string(format!("{:?}", err.kind())));
                            error = true;
                        }
                    }
        
                    if !error {
                        let _ = o_success.send_void(());
                    }
                    
                },
                Err(err) => {
                    let _ = futures::join!(o_failure.send_void(()),o_message.send_string(format!("{:?}", err.kind())));
                }
            }
        }
    
        ResultStatus::Ok
    }
);


