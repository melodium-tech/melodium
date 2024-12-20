
use melodium_core::{common::executive::*, *};
use melodium_macro::{check, mel_treatment};
use std::sync::Arc;
use crate::logger::*;
use colored::Colorize;

#[mel_treatment(
    input logs Stream<Log>
)]
pub async fn console() {
    while let Ok(logs) = logs.recv_many().await.map(|values| Into::<VecDeque<Value>>::into(values)) {
        for log in logs.into_iter().map(|log| GetData::<Arc<dyn Data>>::try_data(log)
        .unwrap()
        .downcast_arc::<Log>()
        .unwrap()) {
            match log.level {
                LogLevel::Error => println!("{}: {}: {}", "error".bold().red(), log.label, log.message),
                LogLevel::Warning => todo!(),
                LogLevel::Info => todo!(),
                LogLevel::Debug => todo!(),
                LogLevel::Trace => todo!(),
            }

            
        }
    }
}


