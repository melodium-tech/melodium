#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use async_std::io::BufReader;
use async_std::process::Command;
use futures::{AsyncReadExt, AsyncWriteExt};
use melodium_core::*;
use melodium_macro::{check, mel_model, mel_package, mel_treatment};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Weak;
use std_mel::data::*;

#[derive(Debug)]
#[mel_model(
    param working_dir Option<string> none
    param clear_env bool false
    param env Map none
)]
pub struct Environment {
    _model: Weak<EnvironmentModel>,
}

impl Environment {
    fn new(_model: Weak<EnvironmentModel>) -> Self {
        Self { _model }
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}
}

#[mel_treatment(
    input args Block<Vec<string>>
    input launch Block<void>
    output started Block<void>
    output ended Block<Option<i32>>
    output success Block<bool>
    output failure Block<string>
    model env Environment
)]
pub async fn exec(command: string) {
    if let (Ok(args), Ok(_)) = (
        args.recv_one()
            .await
            .map(|val| GetData::<Vec<string>>::try_data(val).unwrap()),
        launch.recv_one().await,
    ) {
        let env = EnvironmentModel::into(env);
        let mut command = Command::new(command);

        if env.get_clear_env() {
            command.env_clear();
        }

        if let Some(working_dir) = env.get_working_dir() {
            command.current_dir(working_dir);
        }

        command.envs(env.get_env().map.iter().map(|(k, v)| {
            if v.datatype()
                .implements(&melodium_core::common::descriptor::DataTrait::ToString)
            {
                (k.clone(), melodium_core::DataTrait::to_string(v))
            } else {
                (k.clone(), "".to_string())
            }
        }));

        command.args(args);

        command.stdin(Stdio::null());
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());

        command.kill_on_drop(true);

        match command.spawn() {
            Ok(mut child) => {
                let _ = started.send_one(().into()).await;
                match child.status().await {
                    Ok(status) => {
                        let _ = success.send_one(status.success().into()).await;
                        let _ = ended.send_one(status.code().into()).await;
                    }
                    Err(err) => {
                        let _ = failure.send_one(err.to_string().into()).await;
                    }
                }
            }
            Err(err) => {
                let _ = failure.send_one(err.to_string().into()).await;
            }
        }
    }
}

#[mel_treatment(
    input args Block<Vec<string>>
    input launch Block<void>
    input stdin Stream<byte>
    output started Block<void>
    output stdout Stream<byte>
    output stderr Stream<byte>
    output ended Block<Option<i32>>
    output success Block<bool>
    output failure Block<string>
    model env Environment
)]
pub async fn spawn(command: string) {
    if let (Ok(args), Ok(_)) = (
        args.recv_one()
            .await
            .map(|val| GetData::<Vec<string>>::try_data(val).unwrap()),
        launch.recv_one().await,
    ) {
        let env = EnvironmentModel::into(env);
        let mut command = Command::new(command);

        if env.get_clear_env() {
            command.env_clear();
        }

        if let Some(working_dir) = env.get_working_dir() {
            command.current_dir(working_dir);
        }

        command.envs(env.get_env().map.iter().map(|(k, v)| {
            if v.datatype()
                .implements(&melodium_core::common::descriptor::DataTrait::ToString)
            {
                (k.clone(), melodium_core::DataTrait::to_string(v))
            } else {
                (k.clone(), "".to_string())
            }
        }));

        command.args(args);

        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        command.kill_on_drop(true);

        match command.spawn() {
            Ok(mut child) => {
                let _ = started.send_one(().into()).await;

                let child_stdin = child.stdin.take();
                let child_stdout = child.stdout.take();
                let child_stderr = child.stderr.take();

                let write_stdin = async {
                    if let Some(mut child_stdin) = child_stdin {
                        while let Ok(data) = stdin
                            .recv_many()
                            .await
                            .map(|values| TryInto::<Vec<u8>>::try_into(values).unwrap())
                        {
                            check!(child_stdin.write_all(&data).await);
                            check!(child_stdin.flush().await);
                        }

                        let _ = child_stdin.close().await;
                    } else {
                        let _ = stdin.close();
                    }
                };

                let read_stdout = async {
                    if let Some(child_stdout) = child_stdout {
                        let mut child_stdout = BufReader::new(child_stdout);
                        let mut buffer = [0; 2usize.pow(20)];

                        while let Ok(n) = child_stdout.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(
                                stdout
                                    .send_many(TransmissionValue::Byte(
                                        buffer[..n].iter().cloned().collect()
                                    ))
                                    .await
                            )
                        }
                    } else {
                        let _ = stdout.close().await;
                    }
                };

                let read_stderr = async {
                    if let Some(child_stderr) = child_stderr {
                        let mut child_stderr = BufReader::new(child_stderr);
                        let mut buffer = [0; 2usize.pow(20)];

                        while let Ok(n) = child_stderr.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(
                                stderr
                                    .send_many(TransmissionValue::Byte(
                                        buffer[..n].iter().cloned().collect()
                                    ))
                                    .await
                            )
                        }
                    } else {
                        let _ = stderr.close().await;
                    }
                };

                let status = async {
                    match child.status().await {
                        Ok(status) => {
                            let _ = success.send_one(status.success().into()).await;
                            let _ = ended.send_one(status.code().into()).await;
                        }
                        Err(err) => {
                            let _ = failure.send_one(err.to_string().into()).await;
                        }
                    }
                };

                let _ = futures::join!(status, write_stdin, read_stdout, read_stderr);
            }
            Err(err) => {
                let _ = failure.send_one(err.to_string().into()).await;
            }
        }
    }
}

mel_package!();
