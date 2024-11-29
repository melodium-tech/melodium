use async_trait::async_trait;
use core::fmt::Debug;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::AttachParams, Api, Client};
use melodium_core::common::executive::*;
use melodium_macro::check;
use process_mel::{command::Command, environment::Environment, exec::ExecutorEngine};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

struct KubeExecutor {
    client: Client,
    container: String,
}

#[async_trait]
impl ExecutorEngine for KubeExecutor {
    async fn exec(
        &self,
        command: Arc<Command>,
        environment: Option<Arc<Environment>>,
        started: &Box<dyn Output>,
        ended: &Box<dyn Output>,
        success: &Box<dyn Output>,
        failure: &Box<dyn Output>,
    ) {
        let pod: Api<Pod> = Api::all(self.client.clone());

        let mut full_command = if let Some(environment) = environment {
            let mut env_command = vec!["/usr/bin/env".to_string()];

            if environment.clear_env {
                env_command.push("--ignore-environment".to_string());
            }

            if let Some(dir) = &environment.working_directory {
                env_command.push(format!("--chdir={dir}"));
            }

            for (name, val) in &environment.variables.map {
                env_command.push(format!(
                    "{name}={}",
                    if val
                        .datatype()
                        .implements(&melodium_core::common::descriptor::DataTrait::ToString)
                    {
                        melodium_core::DataTrait::to_string(val)
                    } else {
                        "".to_string()
                    }
                ));
            }

            env_command.push("--split-string".to_string());

            env_command.push(command.command.clone());

            env_command
        } else {
            vec![command.command.clone()]
        };

        full_command.extend(command.arguments.clone());

        match pod
            .exec(
                "testcommand",
                full_command,
                &AttachParams::default().container(self.container.clone()),
            )
            .await
        {
            Ok(mut process) => {
                if let Some(status_waiter) = process.take_status() {
                    let _ = started.send_one(().into()).await;

                    if let Some(status) = status_waiter.await {
                        match status.status.as_ref().map(|s| s.as_str()) {
                            Some("Success") => {
                                let _ = success.send_one((status.code == Some(0)).into()).await;
                                let _ = ended.send_one(status.code.into()).await;
                            }
                            _ => {
                                let _ = failure
                                    .send_one(
                                        status
                                            .reason
                                            .unwrap_or_else(|| "No reason provided".to_string())
                                            .into(),
                                    )
                                    .await;
                            }
                        }
                    } else {
                        let _ = failure
                            .send_one("No output status provided".to_string().into())
                            .await;
                    }
                } else {
                    let _ = failure
                        .send_one("Unable to take status waiter".to_string().into())
                        .await;
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
        let pod: Api<Pod> = Api::all(self.client.clone());

        let mut full_command = if let Some(environment) = environment {
            let mut env_command = vec!["/usr/bin/env".to_string()];

            if environment.clear_env {
                env_command.push("--ignore-environment".to_string());
            }

            if let Some(dir) = &environment.working_directory {
                env_command.push(format!("--chdir={dir}"));
            }

            for (name, val) in &environment.variables.map {
                env_command.push(format!(
                    "{name}={}",
                    if val
                        .datatype()
                        .implements(&melodium_core::common::descriptor::DataTrait::ToString)
                    {
                        melodium_core::DataTrait::to_string(val)
                    } else {
                        "".to_string()
                    }
                ));
            }

            env_command.push("--split-string".to_string());

            env_command.push(command.command.clone());

            env_command
        } else {
            vec![command.command.clone()]
        };

        full_command.extend(command.arguments.clone());

        match pod
            .exec(
                "testcommand",
                full_command,
                &AttachParams::default()
                    .container(self.container.clone())
                    .stdin(true)
                    .stdout(true)
                    .stderr(true),
            )
            .await
        {
            Ok(mut process) => {
                if let (
                    Some(status_waiter),
                    Some(mut process_stdin),
                    Some(process_stdout),
                    Some(process_stderr),
                ) = (
                    process.take_status(),
                    process.stdin(),
                    process.stdout(),
                    process.stderr(),
                ) {
                    let _ = started.send_one(().into()).await;

                    let write_stdin = async {
                        while let Ok(data) = stdin
                            .recv_many()
                            .await
                            .map(|values| TryInto::<Vec<u8>>::try_into(values).unwrap())
                        {
                            check!(process_stdin.write_all(&data).await);
                            check!(process_stdin.flush().await);
                        }
                    };

                    let read_stdout = async {
                        let mut process_stdout = BufReader::new(process_stdout);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = process_stdout.read(&mut buffer[..]).await {
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
                    };

                    let read_stderr = async {
                        let mut process_stderr = BufReader::new(process_stderr);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = process_stderr.read(&mut buffer[..]).await {
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
                    };

                    let status_waiter = async {
                        if let Some(status) = status_waiter.await {
                            match status.status.as_ref().map(|s| s.as_str()) {
                                Some("Success") => {
                                    let _ = success.send_one((status.code == Some(0)).into()).await;
                                    let _ = ended.send_one(status.code.into()).await;
                                }
                                _ => {
                                    let _ = failure
                                        .send_one(
                                            status
                                                .reason
                                                .unwrap_or_else(|| "No reason provided".to_string())
                                                .into(),
                                        )
                                        .await;
                                }
                            }
                        } else {
                            let _ = failure
                                .send_one("No output status provided".to_string().into())
                                .await;
                        }
                    };

                    tokio::join!(write_stdin, read_stdout, read_stderr, status_waiter);
                } else {
                    let _ = failure
                        .send_one(
                            "Unable to take status waiter and process I/O"
                                .to_string()
                                .into(),
                        )
                        .await;
                }
            }
            Err(err) => {
                let _ = failure.send_one(err.to_string().into()).await;
            }
        }
    }
}

impl Debug for KubeExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KubeExecutor").field("client", &"").finish()
    }
}
