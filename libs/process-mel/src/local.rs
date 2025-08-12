use crate::{command::*, environment::*, exec::*};
use async_std::io::BufReader;
#[cfg(feature = "real")]
use async_std::process::Command as ProcessCommand;
use async_trait::async_trait;
use futures::{AsyncReadExt, AsyncWriteExt};
use melodium_macro::{check, mel_function};
use regex::{Captures, Replacer};
use std::{fmt::Debug, process::Stdio, sync::Arc};

struct VarReplacer;

impl Replacer for VarReplacer {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        dst.push_str(std::env::var(&caps[1]).unwrap_or_default().as_str());
    }
}

#[derive(Debug)]
pub struct LocalExecutorEngine {}

impl LocalExecutorEngine {
    #[cfg(feature = "real")]
    fn manage_env(process_command: &mut ProcessCommand, env: &Environment) {
        if env.clear_env {
            process_command.env_clear();
        }

        if let Some(working_dir) = env.working_directory.as_ref() {
            process_command.current_dir(working_dir);
        }

        if env.expand_variables {
            let regex = environment_variable_regex();

            for (name, content) in env.variables.map.iter() {
                let expanded = regex.replace_all(content, VarReplacer).to_string();
                process_command.env(name, expanded);
            }
        } else {
            process_command.envs(&env.variables.map);
        }
    }
}

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
        #[cfg(feature = "real")]
        {
            let mut process_command = ProcessCommand::new(&command.command);

            if let Some(environment) = environment.as_ref() {
                LocalExecutorEngine::manage_env(&mut process_command, environment);
            }

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
        #[cfg(feature = "mock")]
        {
            let _ = failed().await;
            let _ = error("Mock mode".to_string()).await;
            let _ = finished().await;
        }
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
        #[cfg(feature = "real")]
        {
            let mut process_command = ProcessCommand::new(&command.command);

            if let Some(environment) = environment.as_ref() {
                LocalExecutorEngine::manage_env(&mut process_command, environment);
            }

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
        #[cfg(feature = "mock")]
        {
            let _ = failed().await;
            let _ = error("Mock mode".to_string()).await;
            let _ = finished().await;
        }
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
        #[cfg(feature = "real")]
        {
            let mut process_command = ProcessCommand::new(&command.command);

            if let Some(environment) = environment.as_ref() {
                LocalExecutorEngine::manage_env(&mut process_command, environment);
            }

            process_command.args(command.arguments.iter());

            process_command.stdin(Stdio::null());
            process_command.stdout(Stdio::piped());
            process_command.stderr(Stdio::piped());

            process_command.kill_on_drop(true);

            eprintln!("Spawn command: {process_command:?}");

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
                            Ok(status) if status.success() => {
                                completed().await;
                                exit(status.code()).await;
                            }
                            Ok(status) => {
                                failed().await;
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
        #[cfg(feature = "mock")]
        {
            let _ = failed().await;
            let _ = error("Mock mode".to_string()).await;
            let _ = finished().await;
        }
    }
}

#[mel_function]
pub fn local_executor() -> Option<Executor> {
    Some(Executor {
        executor: Arc::new(LocalExecutorEngine {}),
    })
}
