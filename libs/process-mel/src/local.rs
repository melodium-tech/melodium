use crate::{command::*, environment::*, exec::*};
use async_std::io::BufReader;
use async_std::process::Command as ProcessCommand;
use async_trait::async_trait;
use common::executive::{Input, Output};
use futures::{AsyncReadExt, AsyncWriteExt};
use melodium_core::*;
use melodium_macro::{check, mel_function};
use std::{fmt::Debug, process::Stdio, sync::Arc};

#[derive(Debug)]
struct LocalExecutorEngine {}

#[async_trait]
impl ExecutorEngine for LocalExecutorEngine {
    async fn exec(
        &self,
        command: Arc<Command>,
        environment: Option<Arc<Environment>>,
        started: &Box<dyn Output>,
        ended: &Box<dyn Output>,
        success: &Box<dyn Output>,
        failure: &Box<dyn Output>,
    ) {
        let mut process_command = ProcessCommand::new(&command.command);

        if environment
            .as_ref()
            .map(|env| env.clear_env)
            .unwrap_or(false)
        {
            process_command.env_clear();
        }

        if let Some(working_dir) = environment
            .as_ref()
            .map(|env| env.working_directory.as_ref())
            .flatten()
        {
            process_command.current_dir(working_dir);
        }

        process_command.envs(
            environment
                .map(|env| {
                    env.variables
                        .map
                        .iter()
                        .map(|(k, v)| {
                            if v.datatype()
                                .implements(&melodium_core::common::descriptor::DataTrait::ToString)
                            {
                                (k.clone(), melodium_core::DataTrait::to_string(v))
                            } else {
                                (k.clone(), "".to_string())
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        );

        process_command.args(command.arguments.iter());

        process_command.stdin(Stdio::null());
        process_command.stdout(Stdio::null());
        process_command.stderr(Stdio::null());

        process_command.kill_on_drop(true);

        match process_command.spawn() {
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

    async fn spawn(
        &self,
        command: Arc<Command>,
        environment: Option<Arc<Environment>>,
        started: &Box<dyn Output>,
        ended: &Box<dyn Output>,
        success: &Box<dyn Output>,
        failure: &Box<dyn Output>,
        stdin: &Box<dyn Input>,
        stdout: &Box<dyn Output>,
        stderr: &Box<dyn Output>,
    ) {
        let mut process_command = ProcessCommand::new(&command.command);

        if environment
            .as_ref()
            .map(|env| env.clear_env)
            .unwrap_or(false)
        {
            process_command.env_clear();
        }

        if let Some(working_dir) = environment
            .as_ref()
            .map(|env| env.working_directory.as_ref())
            .flatten()
        {
            process_command.current_dir(working_dir);
        }

        process_command.envs(
            environment
                .map(|env| {
                    env.variables
                        .map
                        .iter()
                        .map(|(k, v)| {
                            if v.datatype()
                                .implements(&melodium_core::common::descriptor::DataTrait::ToString)
                            {
                                (k.clone(), melodium_core::DataTrait::to_string(v))
                            } else {
                                (k.clone(), "".to_string())
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        );

        process_command.args(command.arguments.iter());

        process_command.stdin(Stdio::piped());
        process_command.stdout(Stdio::piped());
        process_command.stderr(Stdio::piped());

        process_command.kill_on_drop(true);

        match process_command.spawn() {
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
                        let mut buffer = vec![0; 2usize.pow(20)];

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
                            );
                        }
                    } else {
                        let _ = stdout.close().await;
                    }
                };

                let read_stderr = async {
                    if let Some(child_stderr) = child_stderr {
                        let mut child_stderr = BufReader::new(child_stderr);
                        let mut buffer = vec![0; 2usize.pow(20)];

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
                            );
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

#[mel_function]
pub fn local_executor() -> Option<Executor> {
    Some(Executor {
        executor: Arc::new(LocalExecutorEngine {}),
    })
}
