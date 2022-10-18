
use crate::core::prelude::*;

treatment!(direct_reading_treatment,
    core_identifier!("fs","read";"DirectReading"),
    indoc!(r#"Read one file.

    The content of the file given through `path` is streamed through `data`.
    
    If any reading failure happens, `failure` is emitted and `message` contains text of the related error."#).to_string(),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("path",Scalar,String,Block)
    ],
    outputs![
        output!("data",Scalar,Byte,Stream),
        output!("failure",Scalar,Void,Block),
        output!("message",Scalar,String,Stream)
    ],
    host {
        let i_path = host.get_input("path");

        let o_data = host.get_output("data");
        let o_failure = host.get_output("failure");
        let o_message = host.get_output("message");

        if let Ok(path) = i_path.recv_one_string().await {

            let os_path = async_std::path::PathBuf::from(path);
            let open_result = async_std::fs::File::open(&os_path).await;

            match open_result {
                Ok(mut file) => {

                    let mut buf = vec![0; 1048576];
                    loop {

                        match file.read(&mut buf).await {
                            Ok(n) => {
                                if n == 0 {
                                    break;
                                }

                                ok_or_break!(o_data.send_multiple_byte(buf.get(0..n).unwrap().to_vec()).await);
                            },
                            Err(err) => {

                                let _ = futures::join!(o_failure.send_void(()),o_message.send_string(format!("{:?}", err.kind())));
                            },
                        }
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
