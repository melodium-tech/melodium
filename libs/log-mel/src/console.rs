use crate::logger::*;
use colored::Colorize;
use melodium_core::{common::executive::*, *};
use melodium_macro::mel_treatment;
use std::sync::Arc;

#[mel_treatment(
    input logs Stream<Log>
    default timestamp false
)]
pub async fn console(timestamp: bool) {
    while let Ok(logs) = logs
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        for log in logs.into_iter().map(|log| {
            GetData::<Arc<dyn Data>>::try_data(log)
                .unwrap()
                .downcast_arc::<Log>()
                .unwrap()
        }) {
            match log.level {
                LogLevel::Error => {
                    if timestamp {
                        println!(
                            "[{}] {}: {}: {}",
                            log.timestamp
                                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                            "error".bold().red(),
                            log.label,
                            log.message
                        )
                    } else {
                        println!("{}: {}: {}", "error".bold().red(), log.label, log.message)
                    }
                }
                LogLevel::Warning => {
                    if timestamp {
                        println!(
                            "[{}] {}: {}: {}",
                            log.timestamp
                                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                            "warning".bold().yellow(),
                            log.label,
                            log.message
                        )
                    } else {
                        println!(
                            "{}: {}: {}",
                            "warning".bold().yellow(),
                            log.label,
                            log.message
                        )
                    }
                }
                LogLevel::Info => {
                    if timestamp {
                        println!(
                            "[{}] {}: {}: {}",
                            log.timestamp
                                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                            "info".bold().blue(),
                            log.label,
                            log.message
                        )
                    } else {
                        println!("{}: {}: {}", "info".bold().blue(), log.label, log.message)
                    }
                }
                LogLevel::Debug => {
                    if timestamp {
                        println!(
                            "[{}] {}: {}: {}",
                            log.timestamp
                                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                            "debug".bold().purple(),
                            log.label,
                            log.message
                        )
                    } else {
                        println!(
                            "{}: {}: {}",
                            "debug".bold().purple(),
                            log.label,
                            log.message
                        )
                    }
                }
                LogLevel::Trace => {
                    if timestamp {
                        println!(
                            "[{}] {}: {}: {}",
                            log.timestamp
                                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                            "trace".bold().dimmed(),
                            log.label,
                            log.message
                        )
                    } else {
                        println!(
                            "{}: {}: {}",
                            "trace".bold().dimmed(),
                            log.label,
                            log.message
                        )
                    }
                }
            }
        }
    }
}
