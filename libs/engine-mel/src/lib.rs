//!
//! # Mélodium core engine interaction library
//! 
//! This library provides the engine interaction functions and treatments for the Mélodium environment.
//! 
//! ## For Mélodium project
//! 
//! This library is made for use within the Mélodium environment and has no purpose for pure Rust projects.
//! Please refer to the [Mélodium Project](https://melodium.tech/) for more accurate and detailed information.
//! 

use melodium_macro::{mel_package, mel_model};
use melodium_core::common::executive::{Output, ResultStatus};

/// Provides interactions with Mélodium engine.
/// 
/// `ready` source is triggered at startup when engine is ready to process.
#[mel_model(
    source ready () (trigger Block<void>)
    continuous (continuous)
)]
#[derive(Debug)]
pub struct Engine {
    model: std::sync::Weak<EngineModel>,
}

impl Engine {
    fn new(model: std::sync::Weak<EngineModel>) -> Self {
        Self {
            model
        }
    }

    async fn continuous(&self) {
        let model = self.model.upgrade().unwrap();

        model.new_ready(None, Some(Box::new(|mut outputs| {

            let trigger = outputs.remove("trigger").unwrap();

            vec![Box::new(Box::pin(Self::ready(trigger)))]
            
        }))).await;
    }

    async fn ready(trigger: Box<dyn Output>) -> ResultStatus {
        let _ = trigger.send_one_void(()).await;
        trigger.close().await;
        ResultStatus::Ok
    }
}

mel_package!();
