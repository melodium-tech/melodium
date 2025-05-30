use crate::{command::*, environment::*};
use async_trait::async_trait;
use core::pin::Pin;
use melodium_core::*;
use melodium_macro::{mel_data, mel_treatment};
use std::{fmt::Debug, future::Future, sync::Arc};

pub type OnceTriggerCall<'a> =
    Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send + Sync + 'a>;
pub type OnceMessageCall<'a> =
    Box<dyn FnOnce(String) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send + Sync + 'a>;
pub type OnceCodeCall<'a> = Box<
    dyn FnOnce(Option<i32>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send + Sync + 'a,
>;
pub type InDataCall<'a> = Box<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<Vec<u8>, ()>> + Send + 'a>> + Send + Sync + 'a,
>;
pub type OutDataCall<'a> = Box<
    dyn Fn(VecDeque<u8>) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>>
        + Send
        + Sync
        + 'a,
>;

#[async_trait]
pub trait ExecutorEngine: Debug + Send + Sync {
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
    );
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
    );
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
    );
}

/// Provides execution engine for external commands.
///
/// `Executor` is provided by functions, such as `|local_executor`,
/// and is aimed to be used with the `exec` and `spawn` treatments.
#[derive(Debug, Serialize)]
#[mel_data]
pub struct Executor {
    #[serde(skip)]
    pub executor: Arc<dyn ExecutorEngine>,
}

/// Executes a command.
///
/// Takes an `Executor` on which `command` will be run with the optional `environment`.
///
/// When the execution finishes, `finished` is emitted, regardless of the execution or command status.
/// `completed` is emitted if the command execution went right from executor perspective
/// (the command itself may have failed in its own logic),
/// and `exit` contains the return code of the command. `failed` is emitted if the executor
/// is not able to launch the command, and `error` contains the associated error message.
#[mel_treatment(
    input executor Block<Executor>
    input command Block<Command>
    input environment Block<Option<Environment>>
    output started Block<void>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output exit Block<Option<i32>>
)]
pub async fn exec_one() {
    if let (Ok(executor), Ok(command), Ok(environment)) = (
        executor.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Executor>()
                .unwrap()
        }),
        command.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Command>()
                .unwrap()
        }),
        environment.recv_one().await.map(|val| match val {
            Value::Option(val) => val.map(|val| {
                GetData::<Arc<dyn Data>>::try_data(*val)
                    .unwrap()
                    .downcast_arc::<Environment>()
                    .unwrap()
            }),
            _ => unreachable!(),
        }),
    ) {
        executor
            .executor
            .exec(
                &command,
                environment.as_deref(),
                Box::new(|| {
                    Box::pin(async {
                        let _ = started.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = finished.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = completed.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = failed.send_one(().into()).await;
                    })
                }),
                Box::new(|msg: String| {
                    Box::pin(async {
                        let _ = error.send_one(msg.into()).await;
                    })
                }),
                Box::new(|code: Option<i32>| {
                    Box::pin({
                        let exit = &exit;
                        async move {
                            let _ = exit.send_one(code.into()).await;
                        }
                    })
                }),
            )
            .await;
    }
}

/// Executes commands.
///
/// Takes an `Executor` on which `commands` will be run with the optional `environment`.
///
/// When the execution finishes, `finished` is emitted, regardless of the execution or commands status.
/// `completed` is emitted if all the commands executions went right from executor perspective
/// (the command thelselves may have failed in their own logic),
/// and `exit` contains the return code of each command. `failed` is emitted if the executor
/// is not able to launch a command, and `error` contains the associated error message, and no new command is executed.
#[mel_treatment(
    input executor Block<Executor>
    input commands Stream<Command>
    input environment Block<Option<Environment>>
    output started Block<void>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output exit Stream<Option<i32>>
)]
pub async fn exec() {
    if let (Ok(executor), Ok(environment)) = (
        executor.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Executor>()
                .unwrap()
        }),
        environment.recv_one().await.map(|val| match val {
            Value::Option(val) => val.map(|val| {
                GetData::<Arc<dyn Data>>::try_data(*val)
                    .unwrap()
                    .downcast_arc::<Environment>()
                    .unwrap()
            }),
            _ => unreachable!(),
        }),
    ) {
        let mut first = true;
        let mut success = true;
        while let Ok(command) = commands.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Command>()
                .unwrap()
        }) {
            executor
                .executor
                .exec(
                    &command,
                    environment.as_deref(),
                    Box::new(|| {
                        Box::pin(async {
                            if first {
                                let _ = started.send_one(().into()).await;
                                first = false;
                            }
                        })
                    }),
                    Box::new(|| Box::pin(async {})),
                    Box::new(|| Box::pin(async {})),
                    Box::new(|| {
                        Box::pin(async {
                            success = false;
                        })
                    }),
                    Box::new(|msg: String| {
                        Box::pin(async {
                            let _ = error.send_one(msg.into()).await;
                        })
                    }),
                    Box::new(|code: Option<i32>| {
                        Box::pin({
                            let exit = &exit;
                            async move {
                                let _ = exit.send_one(code.into()).await;
                            }
                        })
                    }),
                )
                .await;
            if !success {
                break;
            }
        }
        if success {
            let _ = completed.send_one(().into()).await;
        } else {
            let _ = failed.send_one(().into()).await;
        }
        let _ = finished.send_one(().into()).await;
    }
}

/// Spawn a command and provides input and outputs to the process.
///
/// Takes an `Executor` on which `command` will be spawned with the optional `environment`.
///
/// `stdin` corresponds to standard input of the related process, `stdout` to the standard output,
/// and `stderr` to the standard error output.
///
/// When the execution finishes, `finished` is emitted, regardless of the execution or command status.
/// `completed` is emitted if the command execution went right from executor perspective
/// (the command itself may have failed in its own logic),
/// and `exit` contains the return code of the command. `failed` is emitted if the executor
/// is not able to launch the command, and `error` contains the associated error message.
#[mel_treatment(
    input executor Block<Executor>
    input command Block<Command>
    input environment Block<Option<Environment>>
    input stdin Stream<byte>
    output started Block<void>
    output stdout Stream<byte>
    output stderr Stream<byte>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output exit Block<Option<i32>>
)]
pub async fn spawn_one() {
    if let (Ok(executor), Ok(command), Ok(environment)) = (
        executor.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Executor>()
                .unwrap()
        }),
        command.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Command>()
                .unwrap()
        }),
        environment.recv_one().await.map(|val| match val {
            Value::Option(val) => val.map(|val| {
                GetData::<Arc<dyn Data>>::try_data(*val)
                    .unwrap()
                    .downcast_arc::<Environment>()
                    .unwrap()
            }),
            _ => unreachable!(),
        }),
    ) {
        executor
            .executor
            .spawn(
                &command,
                environment.as_deref(),
                Box::new(|| {
                    Box::pin(async {
                        let _ = started.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = finished.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = completed.send_one(().into()).await;
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = failed.send_one(().into()).await;
                    })
                }),
                Box::new(|msg: String| {
                    Box::pin(async {
                        let _ = error.send_one(msg.into()).await;
                    })
                }),
                Box::new(|code: Option<i32>| {
                    Box::pin({
                        let exit = &exit;
                        async move {
                            let _ = exit.send_one(code.into()).await;
                        }
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        stdin
                            .recv_many()
                            .await
                            .map(|values| TryInto::<Vec<u8>>::try_into(values).unwrap())
                            .map_err(|_| ())
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        stdin.close();
                    })
                }),
                Box::new(|data: VecDeque<u8>| {
                    Box::pin(async {
                        stdout
                            .send_many(TransmissionValue::Byte(data))
                            .await
                            .map_err(|_| ())
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = stdout.close().await;
                    })
                }),
                Box::new(|data: VecDeque<u8>| {
                    Box::pin(async {
                        stderr
                            .send_many(TransmissionValue::Byte(data))
                            .await
                            .map_err(|_| ())
                    })
                }),
                Box::new(|| {
                    Box::pin(async {
                        let _ = stderr.close().await;
                    })
                }),
            )
            .await;
    }
}

/// Spawn a command and provides input and outputs to the process.
///
/// Takes an `Executor` on which `command` will be spawned with the optional `environment`.
///
/// `stdin` corresponds to standard input of the related process, `stdout` to the standard output,
/// and `stderr` to the standard error output.
///
/// When the execution finishes, `finished` is emitted, regardless of the execution or command status.
/// `completed` is emitted if the command execution went right from executor perspective
/// (the command itself may have failed in its own logic),
/// and `exit` contains the return code of the command. `failed` is emitted if the executor
/// is not able to launch the command, and `error` contains the associated error message.
#[mel_treatment(
    input executor Block<Executor>
    input commands Stream<Command>
    input environment Block<Option<Environment>>
    output started Block<void>
    output stdout Stream<byte>
    output stderr Stream<byte>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output exit Stream<Option<i32>>
)]
pub async fn spawn() {
    if let (Ok(executor), Ok(environment)) = (
        executor.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Executor>()
                .unwrap()
        }),
        environment.recv_one().await.map(|val| match val {
            Value::Option(val) => val.map(|val| {
                GetData::<Arc<dyn Data>>::try_data(*val)
                    .unwrap()
                    .downcast_arc::<Environment>()
                    .unwrap()
            }),
            _ => unreachable!(),
        }),
    ) {
        let mut first = true;
        let mut success = true;
        while let Ok(command) = commands.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Command>()
                .unwrap()
        }) {
            executor
                .executor
                .spawn_out(
                    &command,
                    environment.as_deref(),
                    Box::new(|| {
                        Box::pin(async {
                            if first {
                                let _ = started.send_one(().into()).await;
                                first = false;
                            }
                        })
                    }),
                    Box::new(|| Box::pin(async {})),
                    Box::new(|| Box::pin(async {})),
                    Box::new(|| {
                        Box::pin(async {
                            success = false;
                        })
                    }),
                    Box::new(|msg: String| {
                        Box::pin(async {
                            let _ = error.send_one(msg.into()).await;
                        })
                    }),
                    Box::new(|code: Option<i32>| {
                        Box::pin({
                            let exit = &exit;
                            async move {
                                let _ = exit.send_one(code.into()).await;
                            }
                        })
                    }),
                    Box::new(|data: VecDeque<u8>| {
                        Box::pin(async {
                            stdout
                                .send_many(TransmissionValue::Byte(data))
                                .await
                                .map_err(|_| ())
                        })
                    }),
                    Box::new(|| Box::pin(async {})),
                    Box::new(|data: VecDeque<u8>| {
                        Box::pin(async {
                            stderr
                                .send_many(TransmissionValue::Byte(data))
                                .await
                                .map_err(|_| ())
                        })
                    }),
                    Box::new(|| Box::pin(async {})),
                )
                .await;
            if !success {
                break;
            }
        }
        if success {
            let _ = completed.send_one(().into()).await;
        } else {
            let _ = failed.send_one(().into()).await;
        }
        let _ = finished.send_one(().into()).await;
    }
}
