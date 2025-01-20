use async_std::channel::{unbounded, Receiver, Sender};
use chrono::{DateTime, Utc};
use melodium_core::{common::executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_model, mel_treatment};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[mel_data(traits(Serialize Deserialize Bounded))]
pub struct Level {
    pub level: LogLevel,
}

fn level_bounded_min() -> Level {
    Level {
        level: LogLevel::Trace,
    }
}

fn level_bounded_max() -> Level {
    Level {
        level: LogLevel::Error,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[mel_data(traits(Serialize Deserialize))]
pub struct Log {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub label: String,
    pub message: String,
}

#[derive(Debug)]
#[mel_model(
    source logs () () (
        all Stream<Log>
    )
    continuous (continuous)
    shutdown shutdown
)]
pub struct Logger {
    model: Weak<LoggerModel>,
    sender: Sender<Log>,
    receiver: Receiver<Log>,
}

impl Logger {
    pub fn new(model: Weak<LoggerModel>) -> Self {
        let (sender, receiver) = unbounded();
        Self {
            model,
            sender,
            receiver,
        }
    }

    pub fn sender(&self) -> Sender<Log> {
        self.sender.clone()
    }

    pub async fn continuous(&self) {
        let model = self.model.upgrade().unwrap();
        let receiver = self.receiver.clone();

        model
            .new_logs(
                None,
                &HashMap::new(),
                Some(Box::new(move |mut outputs| {
                    let all = outputs.get("all");

                    vec![Box::new(Box::pin(async move {
                        while let Ok(log) = receiver.recv().await {
                            check!(all.send_one(Value::Data(Arc::new(log))).await)
                        }

                        all.close().await;
                        eprintln!("No more logs");
                        ResultStatus::Ok
                    }))]
                })),
            )
            .await;
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    fn shutdown(&self) {
        self.receiver.close();
    }
}

#[mel_treatment(
    model logger Logger
    input messages Stream<string>
)]
pub async fn log_stream(level: Level, label: string) {
    let sender = LoggerModel::into(logger).inner().sender();

    while let Ok(msg) = messages
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        check!(
            sender
                .send(Log {
                    timestamp: Utc::now(),
                    level: level.level.clone(),
                    label: label.clone(),
                    message: msg
                })
                .await
        )
    }
}

#[mel_treatment(
    model logger Logger
    input message Block<string>
)]
pub async fn log_block(level: Level, label: string) {
    let sender = LoggerModel::into(logger).inner().sender();

    if let Ok(msg) = message
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        let _ = sender
            .send(Log {
                timestamp: Utc::now(),
                level: level.level.clone(),
                label: label.clone(),
                message: msg,
            })
            .await;
    }
}

#[mel_treatment(
    model logger Logger
    input display Stream<D>
    generic D (Display)
)]
pub async fn log_data_stream(level: Level, label: string) {
    let sender = LoggerModel::into(logger).inner().sender();

    while let Ok(val) = display.recv_one().await {
        check!(
            sender
                .send(Log {
                    timestamp: Utc::now(),
                    level: level.level.clone(),
                    label: label.clone(),
                    message: format!("{val}")
                })
                .await
        )
    }
}

#[mel_treatment(
    model logger Logger
    input display Block<D>
    generic D (Display)
)]
pub async fn log_data_block(level: Level, label: string) {
    let sender = LoggerModel::into(logger).inner().sender();

    if let Ok(val) = display.recv_one().await {
        let _ = sender
            .send(Log {
                timestamp: Utc::now(),
                level: level.level.clone(),
                label: label.clone(),
                message: format!("{val}"),
            })
            .await;
    }
}

#[mel_function]
pub fn error() -> Level {
    Level {
        level: LogLevel::Error,
    }
}

#[mel_function]
pub fn warning() -> Level {
    Level {
        level: LogLevel::Warning,
    }
}

#[mel_function]
pub fn info() -> Level {
    Level {
        level: LogLevel::Info,
    }
}

#[mel_function]
pub fn debug() -> Level {
    Level {
        level: LogLevel::Debug,
    }
}

#[mel_function]
pub fn trace() -> Level {
    Level {
        level: LogLevel::Trace,
    }
}
