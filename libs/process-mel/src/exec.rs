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
        ended: &Box<dyn Output>,
        success: &Box<dyn Output>,
        failure: &Box<dyn Output>,
    );
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
    );
}

#[derive(Debug, Serialize)]
#[mel_data]
pub struct Executor {
    #[serde(skip)]
    pub executor: Arc<dyn ExecutorEngine>,
}

#[mel_treatment(
    input executor Block<Executor>
    input launch Block<void>
    output started Block<void>
    output ended Block<Option<i32>>
    output success Block<bool>
    output failure Block<string>
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
            .exec(command, environment, &started, &ended, &success, &failure)
            .await;
    }
}

#[mel_treatment(
    input executor Block<Executor>
    input launch Block<void>
    input stdin Stream<byte>
    output started Block<void>
    output stdout Stream<byte>
    output stderr Stream<byte>
    output ended Block<Option<i32>>
    output success Block<bool>
    output failure Block<string>
)]
pub async fn spawn(command: Command, environment: Option<Environment>) {
    eprintln!("Spawn awaiting");
    if let (Ok(executor), Ok(_)) = (
        executor.recv_one().await.map(|val| {
            GetData::<Arc<dyn Data>>::try_data(val)
                .unwrap()
                .downcast_arc::<Executor>()
                .unwrap()
        }),
        launch.recv_one().await,
    ) {
        eprintln!("Spawn try");
        executor
            .executor
            .spawn(
                command,
                environment,
                &started,
                &ended,
                &success,
                &failure,
                &stdin,
                &stdout,
                &stderr,
            )
            .await;
        eprintln!("Spawn finished")
    } else {
        eprintln!("Spawn aborted")
    }
}
