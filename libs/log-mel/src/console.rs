use crate::logger::*;
use colored::Colorize;use melodium_core::{common::executive::*, *};
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
                    println!("{}: {}: {}", "error".bold().red(), log.label, log.message)
                }
                LogLevel::Warning => println!(
                    "{}: {}: {}",
                    "warning".bold().yellow(),
                    log.label,
                    log.message
                ),
                LogLevel::Info => {
                    println!("{}: {}: {}", "info".bold().blue(), log.label, log.message)
                }
                LogLevel::Debug => println!(
                    "{}: {}: {}",
                    "debug".bold().purple(),
                    log.label,
                    log.message
                ),
                LogLevel::Trace => println!(
                    "{}: {}: {}",
                    "trace".bold().dimmed(),
                    log.label,
                    log.message
                ),
            }
        }
    }
}
