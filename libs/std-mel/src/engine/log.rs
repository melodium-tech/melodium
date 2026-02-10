use crate::engine::*;
use melodium_core::common::executive::Level as LogLevel;
use melodium_macro::{mel_data, mel_function, mel_treatment};

#[mel_treatment(
    model engine Engine
    input messages Stream<string>
)]
pub async fn log_stream(level: Level, label: string) {
    let engine = EngineModel::into(engine);

    while let Ok(msgs) = messages
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        for msg in msgs {
            engine
                .world()
                .log(level.level, label.clone(), msg, Some(track_id))
                .await;
        }
    }
}

#[mel_treatment(
    model engine Engine
    input label Block<string>
    input messages Stream<string>
)]
pub async fn log_stream_label(level: Level) {
    let engine = EngineModel::into(engine);

    if let Ok(label) = label
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        while let Ok(msgs) = messages
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
        {
            for msg in msgs {
                engine
                    .world()
                    .log(level.level, label.clone(), msg, Some(track_id))
                    .await;
            }
        }
    }
}

#[mel_treatment(
    model engine Engine
    input message Block<string>
)]
pub async fn log_block(level: Level, label: string) {
    let engine = EngineModel::into(engine);

    if let Ok(msg) = message
        .recv_one()
        .await
        .map(|val| GetData::<string>::try_data(val).unwrap())
    {
        engine
            .world()
            .log(level.level, label, msg, Some(track_id))
            .await;
    }
}

#[mel_treatment(
    model engine Engine
    input label Block<string>
    input message Block<string>
)]
pub async fn log_block_label(level: Level) {
    let engine = EngineModel::into(engine);

    if let Ok(label) = label
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        if let Ok(msg) = message
            .recv_one()
            .await
            .map(|val| GetData::<string>::try_data(val).unwrap())
        {
            engine
                .world()
                .log(level.level, label, msg, Some(track_id))
                .await;
        }
    }
}

#[mel_treatment(
    model engine Engine
    input display Stream<D>
    generic D (Display)
)]
pub async fn log_data_stream(level: Level, label: string) {
    let engine = EngineModel::into(engine);

    while let Ok(values) = display
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        for val in values {
            engine
                .world()
                .log(level.level, label.clone(), format!("{val}"), Some(track_id))
                .await;
        }
    }
}

#[mel_treatment(
    model engine Engine
    input label Block<string>
    input display Stream<D>
    generic D (Display)
)]
pub async fn log_data_stream_label(level: Level) {
    let engine = EngineModel::into(engine);

    if let Ok(label) = label
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        while let Ok(values) = display
            .recv_many()
            .await
            .map(|values| Into::<VecDeque<Value>>::into(values))
        {
            for val in values {
                engine
                    .world()
                    .log(level.level, label.clone(), format!("{val}"), Some(track_id))
                    .await;
            }
        }
    }
}

#[mel_treatment(
    model engine Engine
    input display Block<D>
    generic D (Display)
)]
pub async fn log_data_block(level: Level, label: string) {
    let engine = EngineModel::into(engine);

    if let Ok(val) = display.recv_one().await {
        engine
            .world()
            .log(level.level, label, format!("{val}"), Some(track_id))
            .await;
    }
}

#[mel_treatment(
    model engine Engine
    input label Block<string>
    input display Block<D>
    generic D (Display)
)]
pub async fn log_data_block_label(level: Level) {
    let engine = EngineModel::into(engine);

    if let Ok(label) = label
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        if let Ok(val) = display.recv_one().await {
            engine
                .world()
                .log(level.level, label, format!("{val}"), Some(track_id))
                .await;
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
