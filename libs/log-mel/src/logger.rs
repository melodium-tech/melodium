use async_std::{
    channel::{bounded, unbounded, Receiver, Sender},
    sync::Mutex,
};
use chrono::{DateTime, Utc};
use core::sync::atomic::{AtomicBool, Ordering};
use melodium_core::{common::executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_model, mel_treatment};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry as HashMapEntry, HashMap},
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

#[derive(Debug, Clone)]
struct TrackLogEntry {
    pub track_sender: Sender<Arc<Log>>,
    pub track_receiver: Receiver<Arc<Log>>,
    pub track_receiver_reading_mark: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
struct TrackLog {
    pub common: Sender<Arc<Log>>,
    pub track_sender: Sender<Arc<Log>>,
    pub track_receiver_reading_mark: Arc<AtomicBool>,
    track_id: usize,
    model: Weak<LoggerModel>,
}

impl TrackLog {
    pub async fn send(&self, msg: Arc<Log>) -> Result<(), ()> {
        match (
            self.common.send(Arc::clone(&msg)).await,
            self.track_sender.try_send(Arc::clone(&msg)),
        ) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(track_err)) => {
                match track_err {
                    async_std::channel::TrySendError::Full(_) => {
                        if self.track_receiver_reading_mark.load(Ordering::Relaxed) {
                            let _ = self.track_sender.send(msg).await;
                        } else {
                            self.track_sender.close();
                            if let Some(model) = self.model.upgrade() {
                                model.inner().remove_track(self.track_id).await;
                            }
                        }
                    }
                    async_std::channel::TrySendError::Closed(_) => {}
                }
                Ok(())
            }
            (Err(_), Ok(_)) => Ok(()),
            (Err(_), Err(track_err)) => match track_err {
                async_std::channel::TrySendError::Full(_) => {
                    if self.track_receiver_reading_mark.load(Ordering::Relaxed) {
                        self.track_sender.send(msg).await.map_err(|_| ())
                    } else {
                        self.track_sender.close();
                        if let Some(model) = self.model.upgrade() {
                            model.inner().remove_track(self.track_id).await;
                        }
                        Err(())
                    }
                }
                async_std::channel::TrySendError::Closed(_) => Err(()),
            },
        }
    }
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
    sender: Sender<Arc<Log>>,
    receiver: Receiver<Arc<Log>>,
    tracks: Mutex<HashMap<usize, TrackLogEntry>>,
}

impl Logger {
    pub fn new(model: Weak<LoggerModel>) -> Self {
        let (sender, receiver) = unbounded();
        Self {
            model,
            sender,
            receiver,
            tracks: Mutex::new(HashMap::new()),
        }
    }

    pub(self) async fn senders(&self, track_id: usize) -> TrackLog {
        match self.tracks.lock().await.entry(track_id) {
            HashMapEntry::Occupied(occupied_entry) => TrackLog {
                common: self.sender.clone(),
                track_sender: occupied_entry.get().track_sender.clone(),
                track_receiver_reading_mark: occupied_entry
                    .get()
                    .track_receiver_reading_mark
                    .clone(),
                model: self.model.clone(),
                track_id,
            },
            HashMapEntry::Vacant(vacant_entry) => {
                let couple = bounded(500);
                let entry = vacant_entry.insert(TrackLogEntry {
                    track_sender: couple.0,
                    track_receiver: couple.1,
                    track_receiver_reading_mark: Arc::new(AtomicBool::new(false)),
                });
                TrackLog {
                    common: self.sender.clone(),
                    track_sender: entry.track_sender.clone(),
                    track_receiver_reading_mark: entry.track_receiver_reading_mark.clone(),
                    model: self.model.clone(),
                    track_id,
                }
            }
        }
    }

    pub async fn receiver(&self, track_id: usize) -> Receiver<Arc<Log>> {
        match self.tracks.lock().await.entry(track_id) {
            HashMapEntry::Occupied(occupied_entry) => {
                occupied_entry
                    .get()
                    .track_receiver_reading_mark
                    .store(true, Ordering::Relaxed);
                occupied_entry.get().track_receiver.clone()
            }
            HashMapEntry::Vacant(vacant_entry) => {
                let couple = bounded(500);
                let entry = vacant_entry.insert(TrackLogEntry {
                    track_sender: couple.0,
                    track_receiver: couple.1,
                    track_receiver_reading_mark: Arc::new(AtomicBool::new(true)),
                });
                entry.track_receiver.clone()
            }
        }
    }

    pub async fn remove_track(&self, track_id: usize) {
        let _ = self.tracks.lock().await.remove(&track_id);
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
                            eprintln!("Sending log");
                            check!(all.send_one(Value::Data(log)).await)
                        }

                        all.close().await;
                        eprintln!("Log finished");
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
    input trigger Block<void>
    input stop Block<void>
    output logs Stream<Log>
)]
pub async fn track_logs() {
    let logger = LoggerModel::into(logger);

    if let Ok(_) = trigger.recv_one().await {
        let receiver = logger.inner().receiver(track_id).await;
        let transmit = async {
            while let Ok(log) = receiver.recv().await {
                check!(logs.send_one(Value::Data(log)).await)
            }
            logger.inner().remove_track(track_id).await;
        };
        let stop = async {
            if stop.recv_one().await.is_ok() {
                receiver.close();
            }
        };

        futures::join!(transmit, stop);
    }
}

#[mel_treatment(
    model logger Logger
    input logs Stream<Log>
)]
pub async fn inject_stream_log() {
    let senders = LoggerModel::into(logger).inner().senders(track_id).await;

    while let Ok(log) = logs.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<Log>()
            .unwrap()
    }) {
        check!(senders.send(log).await)
    }
}

#[mel_treatment(
    model logger Logger
    input log Block<Log>
)]
pub async fn inject_block_log() {
    let senders = LoggerModel::into(logger).inner().senders(track_id).await;

    if let Ok(log) = log.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<Log>()
            .unwrap()
    }) {
        let _ = senders.send(log).await;
    }
}

#[mel_treatment(
    model logger Logger
    input messages Stream<string>
    output ended Block<void>
)]
pub async fn log_stream(level: Level, label: string) {
    let senders = LoggerModel::into(logger).inner().senders(track_id).await;

    while let Ok(msg) = messages
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        check!(
            senders
                .send(Arc::new(Log {
                    timestamp: Utc::now(),
                    level: level.level.clone(),
                    label: label.clone(),
                    message: msg
                }))
                .await
        )
    }
    let _ = ended.send_one(().into()).await;
}

#[mel_treatment(
    model logger Logger
    input message Block<string>
    output ended Block<void>
)]
pub async fn log_block(level: Level, label: string) {
    let senders = LoggerModel::into(logger).inner().senders(track_id).await;

    if let Ok(msg) = message
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        let _ = senders
            .send(Arc::new(Log {
                timestamp: Utc::now(),
                level: level.level.clone(),
                label: label.clone(),
                message: msg,
            }))
            .await;
    }
    let _ = ended.send_one(().into()).await;
}

#[mel_treatment(
    model logger Logger
    input display Stream<D>
    output ended Block<void>
    generic D (Display)
)]
pub async fn log_data_stream(level: Level, label: string) {
    let senders = LoggerModel::into(logger).inner().senders(track_id).await;

    while let Ok(val) = display.recv_one().await {
        check!(
            senders
                .send(Arc::new(Log {
                    timestamp: Utc::now(),
                    level: level.level.clone(),
                    label: label.clone(),
                    message: format!("{val}")
                }))
                .await
        )
    }
    let _ = ended.send_one(().into()).await;
}

#[mel_treatment(
    model logger Logger
    input display Block<D>
    output ended Block<void>
    generic D (Display)
)]
pub async fn log_data_block(level: Level, label: string) {
    let senders = LoggerModel::into(logger).inner().senders(track_id).await;

    if let Ok(val) = display.recv_one().await {
        let _ = senders
            .send(Arc::new(Log {
                timestamp: Utc::now(),
                level: level.level.clone(),
                label: label.clone(),
                message: format!("{val}"),
            }))
            .await;
    }
    let _ = ended.send_one(().into()).await;
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
