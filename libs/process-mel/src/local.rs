use crate::{command::*, environment::*, exec::*};
use async_std::io::BufReader;
use async_std::process::Command as ProcessCommand;
use async_trait::async_trait;
use futures::{AsyncReadExt, AsyncWriteExt};
use melodium_macro::{check, mel_function};
use std::{fmt::Debug, process::Stdio, sync::Arc};

#[derive(Debug)]
struct LocalExecutorEngine {}

#[async_trait]
impl ExecutorEngine for LocalExecutorEngine {
    async fn exec(
        &self,
        command: &Command,
        environment: Option<&Environment>,
        started: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
        exit: OnceCodeCall<'async_trait>,
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
                .as_ref()
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
                started().await;
                match child.status().await {
                    Ok(status) => {
                        completed().await;
                        exit(status.code()).await;
                    }
                    Err(err) => {
                        failed().await;
                        error(err.to_string()).await;
                    }
                }
            }
            Err(err) => {
                failed().await;
                error(err.to_string()).await;
            }
        }
        finished().await;
    }

    async fn spawn(
        &self,
        command: &Command,
        environment: Option<&Environment>,
        started: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
        exit: OnceCodeCall<'async_trait>,
        stdin: InDataCall<'async_trait>,
        stdinclose: OnceTriggerCall<'async_trait>,
        stdout: OutDataCall<'async_trait>,
        stdoutclose: OnceTriggerCall<'async_trait>,
        stderr: OutDataCall<'async_trait>,
        stderrclose: OnceTriggerCall<'async_trait>,
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
                started().await;

                let child_stdin = child.stdin.take();
                let child_stdout = child.stdout.take();
                let child_stderr = child.stderr.take();

                let write_stdin = async {
                    if let Some(mut child_stdin) = child_stdin {
                        while let Ok(data) = stdin().await {
                            check!(child_stdin.write_all(&data).await);
                            check!(child_stdin.flush().await);
                        }

                        let _ = child_stdin.close().await;
                    } else {
                        stdinclose().await;
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
                            check!(stdout(buffer[..n].iter().cloned().collect()).await);
                        }
                    } else {
                        stdoutclose().await;
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
                            check!(stderr(buffer[..n].iter().cloned().collect()).await);
                        }
                    } else {
                        stderrclose().await;
                    }
                };

                let status = async {
                    match child.status().await {
                        Ok(status) => {
                            completed().await;
                            exit(status.code()).await;
                        }
                        Err(err) => {
                            failed().await;
                            error(err.to_string()).await;
                        }
                    }
                };

                let _ = futures::join!(status, write_stdin, read_stdout, read_stderr);
            }
            Err(err) => {
                failed().await;
                error(err.to_string()).await;
            }
        }
        finished().await;
    }

    async fn spawn_out(
        &self,
        command: &Command,
        environment: Option<&Environment>,
        started: OnceTriggerCall<'async_trait>,
        finished: OnceTriggerCall<'async_trait>,
        completed: OnceTriggerCall<'async_trait>,
        failed: OnceTriggerCall<'async_trait>,
        error: OnceMessageCall<'async_trait>,
        exit: OnceCodeCall<'async_trait>,
        stdout: OutDataCall<'async_trait>,
        stdoutclose: OnceTriggerCall<'async_trait>,
        stderr: OutDataCall<'async_trait>,
        stderrclose: OnceTriggerCall<'async_trait>,
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
        process_command.stdout(Stdio::piped());
        process_command.stderr(Stdio::piped());

        process_command.kill_on_drop(true);

        match process_command.spawn() {
            Ok(mut child) => {
                started().await;

                let child_stdout = child.stdout.take();
                let child_stderr = child.stderr.take();

                let read_stdout = async {
                    if let Some(child_stdout) = child_stdout {
                        let mut child_stdout = BufReader::new(child_stdout);
                        let mut buffer = vec![0; 2usize.pow(20)];

                        while let Ok(n) = child_stdout.read(&mut buffer[..]).await {
                            if n == 0 {
                                break;
                            }
                            check!(stdout(buffer[..n].iter().cloned().collect()).await);
                        }
                    } else {
                        stdoutclose().await;
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
                            check!(stderr(buffer[..n].iter().cloned().collect()).await);
                        }
                    } else {
                        stderrclose().await;
                    }
                };

                let status = async {
                    match child.status().await {
                        Ok(status) => {
                            completed().await;
                            exit(status.code()).await;
                        }
                        Err(err) => {
                            failed().await;
                            error(err.to_string()).await;
                        }
                    }
                };

                let _ = futures::join!(status, read_stdout, read_stderr);
            }
            Err(err) => {
                failed().await;
                error(err.to_string()).await;
            }
        }
        finished().await;
    }
}

#[mel_function]
pub fn local_executor() -> Option<Executor> {
    Some(Executor {
        executor: Arc::new(LocalExecutorEngine {}),
    })
}
