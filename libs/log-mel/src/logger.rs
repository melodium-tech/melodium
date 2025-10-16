use async_std::{
    channel::{bounded, unbounded, Receiver, Sender},
    sync::RwLock,
};
use chrono::{DateTime, Utc};
use core::{
    fmt::Display,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};
use futures::{pin_mut, select, FutureExt};
use melodium_core::{common::executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_model, mel_treatment};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

static LOG_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warning => write!(f, "warning"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[mel_data(traits(Serialize Deserialize Bounded PartialEquality Equality PartialOrder Order))]
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
#[mel_data(traits(Serialize Deserialize ToString))]
pub struct Log {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub label: String,
    pub message: String,
    #[serde(deserialize_with = "deserialize_increment_log_count")]
    _count: (),
}

impl Log {
    pub fn new(timestamp: DateTime<Utc>, level: LogLevel, label: String, message: String) -> Self {
        LOG_COUNT.fetch_add(1, Ordering::Relaxed);
        Self {
            timestamp,
            level,
            label,
            message,
            _count: (),
        }
    }
}

impl Drop for Log {
    fn drop(&mut self) {
        LOG_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

fn deserialize_increment_log_count<'de, D>(_deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    LOG_COUNT.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

impl melodium_core::executive::ToString for Log {
    fn to_string(&self) -> String {
        format!(
            "[{}] {}: {}: {}",
            self.timestamp
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            self.level,
            self.label,
            self.message
        )
    }
}

/*
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
}*/

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
    sender: Sender<VecDeque<Arc<Log>>>,
    receiver: Receiver<VecDeque<Arc<Log>>>,
    senders: RwLock<Vec<Sender<VecDeque<Arc<Log>>>>>,
    entries_open: RwLock<Vec<Arc<AtomicBool>>>,
    /*tracks: Mutex<HashMap<usize, TrackLogEntry>>,
    inner_stop_barrier: Arc<Barrier>,
    common_stop_barrier: Mutex<Option<Arc<Barrier>>>,
    immediate_stop: Arc<AtomicBool>,*/
}

impl Logger {
    pub fn new(model: Weak<LoggerModel>) -> Self {
        let (sender, receiver) = bounded(1);
        //let barrier = Arc::new(Barrier::new(2));
        Self {
            model,
            sender,
            receiver,
            senders: RwLock::new(Vec::new()),
            entries_open: RwLock::new(Vec::new()),
            /*tracks: Mutex::new(HashMap::new()),
            inner_stop_barrier: Arc::clone(&barrier),
            common_stop_barrier: Mutex::new(Some(barrier)),
            immediate_stop: Arc::new(AtomicBool::new(false)),*/
        }
    }

    pub(crate) async fn add_track_log(&self, sender: Sender<VecDeque<Arc<Log>>>) {
        self.senders.write().await.push(sender);
    }

    pub(crate) async fn add_entry_open(&self, entry_open: Arc<AtomicBool>) {
        self.entries_open.write().await.push(entry_open);
    }

    pub async fn send_logs(&self, logs: VecDeque<Arc<Log>>) -> bool {
        match self.sender.send(logs).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub async fn close_common(&self) {
        let model = self.model.upgrade().unwrap();
        async_std::task::spawn(async move {
            loop {
                if !model
                    .inner()
                    .entries_open
                    .read()
                    .await
                    .iter()
                    .any(|entry| entry.load(Ordering::Relaxed))
                {
                    break;
                }
                async_std::task::sleep(Duration::from_millis(50)).await;
            }

            model.inner().receiver.close();
        });
    }

    #[cfg(feature = "real")]
    pub async fn continuous(&self) {
        let model = self.model.upgrade().unwrap();

        model
            .clone()
            .new_logs(
                None,
                &HashMap::new(),
                Some(Box::new(move |mut outputs| {
                    let all = outputs.get("all");
                    vec![Box::new(Box::pin(async move {
                        let (sender, receiver) = bounded(1);
                        model.inner().add_track_log(sender).await;

                        while let Ok(logs) = receiver.recv().await {
                            check!(
                                all.send_many(TransmissionValue::Other(
                                    logs.into_iter().map(|log| Value::Data(log)).collect()
                                ))
                                .await
                            );
                            all.force_send().await;
                        }

                        all.close().await;
                        ResultStatus::Ok
                    }))]
                })),
            )
            .await;

        while let Ok(logs) = self.receiver.recv().await {
            let senders = self.senders.read().await;
            for sender in senders.iter() {
                let _ = sender.send(logs.clone()).await;
            }
        }

        let senders = self.senders.read().await;
        for sender in senders.iter() {
            let _ = sender.close();
        }
    }

    #[cfg(feature = "mock")]
    pub async fn continuous(&self) {}

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    fn shutdown(&self) {
        self.receiver.close();
    }
}

#[mel_treatment(
    model logger Logger
    input trigger Block<void>
)]
pub async fn stop() {
    let logger = LoggerModel::into(logger);

    if let Ok(_) = trigger.recv_one().await {
        logger.inner().close_common().await;
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
        let (sender, receiver) = bounded(1);

        logger.inner().add_track_log(sender).await;

        let stop = async {
            if let Ok(_) = stop.recv_one().await {
                receiver.close();
                true
            } else {
                false
            }
        }
        .fuse();

        let transmit = async {
            while let Ok(values) = receiver.recv().await {
                check!(
                    logs.send_many(TransmissionValue::Other(
                        values.into_iter().map(|val| Value::Data(val)).collect()
                    ))
                    .await
                );
                logs.force_send().await;
            }
        }
        .fuse();

        pin_mut!(stop, transmit);

        loop {
            select! {
                effective = stop => { if effective { break } }
                () = transmit => { break }
                complete => break,
            }
        }
    }
}

#[mel_treatment(
    model logger Logger
    input logs Stream<Log>
)]
pub async fn inject_stream_log() {
    let logger = LoggerModel::into(logger);

    let entry = Arc::new(AtomicBool::new(true));
    logger.inner().add_entry_open(entry.clone()).await;

    while let Ok(logs) = logs
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        if !logger
            .inner()
            .send_logs(
                logs.into_iter()
                    .map(|val| {
                        GetData::<Arc<dyn Data>>::try_data(val)
                            .unwrap()
                            .downcast_arc::<Log>()
                            .unwrap()
                    })
                    .collect(),
            )
            .await
        {
            break;
        }
    }

    entry.store(false, Ordering::Relaxed);
}

#[mel_treatment(
    model logger Logger
    input log Block<Log>
)]
pub async fn inject_block_log() {
    let logger = LoggerModel::into(logger);

    let entry = Arc::new(AtomicBool::new(true));
    logger.inner().add_entry_open(entry.clone()).await;

    if let Ok(log) = log.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<Log>()
            .unwrap()
    }) {
        logger.inner().send_logs(vec![log].into()).await;
    }

    entry.store(false, Ordering::Relaxed);
}

#[mel_treatment(
    model logger Logger
    input messages Stream<string>
    output ended Block<void>
)]
pub async fn log_stream(level: Level, label: string) {
    let logger = LoggerModel::into(logger);

    let entry = Arc::new(AtomicBool::new(true));
    logger.inner().add_entry_open(entry.clone()).await;

    while let Ok(msgs) = messages
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        if !logger
            .inner()
            .send_logs(
                msgs.into_iter()
                    .map(|msg| {
                        Arc::new(Log::new(
                            Utc::now(),
                            level.level.clone(),
                            label.clone(),
                            msg,
                        ))
                    })
                    .collect(),
            )
            .await
        {
            break;
        }
    }
    entry.store(false, Ordering::Relaxed);

    let _ = ended.send_one(().into()).await;
}

#[mel_treatment(
    model logger Logger
    input message Block<string>
    output ended Block<void>
)]
pub async fn log_block(level: Level, label: string) {
    let logger = LoggerModel::into(logger);

    let entry = Arc::new(AtomicBool::new(true));
    logger.inner().add_entry_open(entry.clone()).await;

    if let Ok(msg) = message
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        logger
            .inner()
            .send_logs(
                vec![Arc::new(Log::new(
                    Utc::now(),
                    level.level.clone(),
                    label.clone(),
                    msg,
                ))]
                .into(),
            )
            .await;
    }
    entry.store(false, Ordering::Relaxed);

    let _ = ended.send_one(().into()).await;
}

#[mel_treatment(
    model logger Logger
    input display Stream<D>
    output ended Block<void>
    generic D (Display)
)]
pub async fn log_data_stream(level: Level, label: string) {
    let logger = LoggerModel::into(logger);

    let entry = Arc::new(AtomicBool::new(true));
    logger.inner().add_entry_open(entry.clone()).await;

    while let Ok(values) = display
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        if !logger
            .inner()
            .send_logs(
                values
                    .into_iter()
                    .map(|val| {
                        Arc::new(Log::new(
                            Utc::now(),
                            level.level.clone(),
                            label.clone(),
                            format!("{val}"),
                        ))
                    })
                    .collect(),
            )
            .await
        {
            break;
        }
    }
    entry.store(false, Ordering::Relaxed);

    let _ = ended.send_one(().into()).await;
}

#[mel_treatment(
    model logger Logger
    input display Block<D>
    output ended Block<void>
    generic D (Display)
)]
pub async fn log_data_block(level: Level, label: string) {
    let logger = LoggerModel::into(logger);

    let entry = Arc::new(AtomicBool::new(true));
    logger.inner().add_entry_open(entry.clone()).await;

    if let Ok(val) = display.recv_one().await {
        logger
            .inner()
            .send_logs(
                vec![Arc::new(Log::new(
                    Utc::now(),
                    level.level.clone(),
                    label.clone(),
                    format!("{val}"),
                ))]
                .into(),
            )
            .await;
    }
    entry.store(false, Ordering::Relaxed);

    let _ = ended.send_one(().into()).await;
}

#[mel_treatment(
    input logs Stream<Log>
    output filtered Stream<Log>
)]
pub async fn filter_logs(levels: Vec<Level>, labels: Vec<string>) {
    let levels = levels.into_iter().map(|lvl| lvl.level).collect::<Vec<_>>();

    while let Ok(logs) = logs
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            filtered
                .send_many(TransmissionValue::Other(
                    logs.into_iter()
                        .map(|log| GetData::<Arc<dyn Data>>::try_data(log)
                            .unwrap()
                            .downcast_arc::<Log>()
                            .unwrap())
                        .filter(|log| levels.contains(&log.level) && labels.contains(&log.label))
                        .map(|log| Value::Data(log))
                        .collect()
                ))
                .await
        )
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
