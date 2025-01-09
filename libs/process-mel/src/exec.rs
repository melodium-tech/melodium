use crate::{command::*, environment::*};
use async_trait::async_trait;
use common::executive::{Input, Output};
use melodium_core::*;
use melodium_macro::{mel_data, mel_treatment};
use std::{fmt::Debug, sync::Arc};

#[async_trait]
pub trait ExecutorEngine: Debug + Send + Sync {
    async fn exec(
        &self,
        command: Arc<Command>,
        environment: Option<Arc<Environment>>,
        started: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        completed: &Box<dyn Output>,
        failed: &Box<dyn Output>,
        error: &Box<dyn Output>,
        exit: &Box<dyn Output>,
    );
    async fn spawn(
        &self,
        command: Arc<Command>,
        environment: Option<Arc<Environment>>,
        started: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        completed: &Box<dyn Output>,
        failed: &Box<dyn Output>,
        error: &Box<dyn Output>,
        exit: &Box<dyn Output>,
        stdin: &Box<dyn Input>,
        stdout: &Box<dyn Output>,
        stderr: &Box<dyn Output>,
    );
    async fn exec_list(
        &self,
        commands: Vec<Arc<Command>>,
        environment: Option<Arc<Environment>>,
        started: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        completed: &Box<dyn Output>,
        failed: &Box<dyn Output>,
        error: &Box<dyn Output>,
        exits: &Box<dyn Output>,
    );
    async fn spawn_list(
        &self,
        commands: Vec<Arc<Command>>,
        environment: Option<Arc<Environment>>,
        started: &Box<dyn Output>,
        finished: &Box<dyn Output>,
        completed: &Box<dyn Output>,
        failed: &Box<dyn Output>,
        error: &Box<dyn Output>,
        exits: &Box<dyn Output>,
        stdout: &Box<dyn Output>,
        stderr: &Box<dyn Output>,
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
/// Takes an `Executor` on which `command` will be run with the optionnal `environment`.
///
/// When the execution finishes, `finished` is emitted, regardless of the execution or command status.
/// `completed` is emitted if the command execution went right from executor perspective
/// (the command itself may have failed in its own logic),
/// and `exit` contains the return code of the command. `failed` is emitted if the executor
/// is not able to launch the command, and `error` contains the associated error message.
#[mel_treatment(
    input executor Block<Executor>
    input launch Block<void>
    output started Block<void>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output exit Block<Option<i32>>
)]
pub async fn exec(command: Command, environment: Option<Environment>) {
    if let (Ok(executor), Ok(_)) = (
        executor.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Executor>()
                .unwrap()
        }),
        launch.recv_one().await,
    ) {
        executor
            .executor
            .exec(
                command,
                environment,
                &started,
                &finished,
                &completed,
                &failed,
                &error,
                &exit,
            )
            .await;
    }
}

/// Spawn a command and provides input and outputs to the process.
///
/// Takes an `Executor` on which `command` will be spawned with the optionnal `environment`.
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
    input launch Block<void>
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
pub async fn spawn(command: Command, environment: Option<Environment>) {
    if let (Ok(executor), Ok(_)) = (
        executor.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Executor>()
                .unwrap()
        }),
        launch.recv_one().await,
    ) {
        executor
            .executor
            .spawn(
                command,
                environment,
                &started,
                &finished,
                &completed,
                &failed,
                &error,
                &exit,
                &stdin,
                &stdout,
                &stderr,
            )
            .await;
    }
}

/// Executes a list of commands.
///
/// Takes an `Executor` on which `commands` will be run with the optionnal `environment`.
///
/// When the execution finishes, `finished` is emitted, regardless of the execution or commands status.
/// `completed` is emitted if all the commands executions went right from executor perspective
/// (the commands themselves may have failed in their own logic),
/// and `exits` contains the return code of each of the commands. `failed` is emitted if the executor
/// is not able to launch a command, and `error` contains the associated error message.
#[mel_treatment(
    input executor Block<Executor>
    input launch Block<void>
    output started Block<void>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output exits Stream<Option<i32>>
)]
pub async fn exec_list(commands: Vec<Command>, environment: Option<Environment>) {
    if let (Ok(executor), Ok(_)) = (
        executor.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Executor>()
                .unwrap()
        }),
        launch.recv_one().await,
    ) {
        executor
            .executor
            .exec_list(
                commands,
                environment,
                &started,
                &finished,
                &completed,
                &failed,
                &error,
                &exits,
            )
            .await;
    }
}

/// Spawn a list of commands and provides outputs of the processes.
///
/// Takes an `Executor` on which `commands` will be spawned with the optionnal `environment`.
///
/// `stdout` corresponds to standard outputs of the related process,
/// and `stderr` to the standard error output.
///
/// When the execution finishes, `finished` is emitted, regardless of the execution or commands status.
/// `completed` is emitted if all the commands executions went right from executor perspective
/// (the commands themselves may have failed in their own logic),
/// and `exits` contains the return code of the commands. `failed` is emitted if the executor
/// is not able to launch a command, and `error` contains the associated error message.
#[mel_treatment(
    input executor Block<Executor>
    input launch Block<void>
    output started Block<void>
    output stdout Stream<byte>
    output stderr Stream<byte>
    output finished Block<void>
    output completed Block<void>
    output failed Block<void>
    output error Block<string>
    output exits Stream<Option<i32>>
)]
pub async fn spawn_list(commands: Vec<Command>, environment: Option<Environment>) {
    if let (Ok(executor), Ok(_)) = (
        executor.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Executor>()
                .unwrap()
        }),
        launch.recv_one().await,
    ) {
        executor
            .executor
            .spawn_list(
                commands,
                environment,
                &started,
                &finished,
                &completed,
                &failed,
                &error,
                &exits,
                &stdout,
                &stderr,
            )
            .await;
    }
}
