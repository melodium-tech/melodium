use melodium_core::{
    common::executive::{Output, ResultStatus},
    *,
};
use melodium_macro::mel_model;
use std::collections::HashMap;

/// Provides interactions with Mélodium engine.
///
/// `ready` source is triggered at startup when engine is ready to process.
#[mel_model(
    source ready () () (trigger Block<void>)
    continuous (continuous)
)]
#[derive(Debug)]
pub struct Engine {
    model: std::sync::Weak<EngineModel>,
}

impl Engine {
    fn new(model: std::sync::Weak<EngineModel>) -> Self {
        Self { model }
    }

    async fn continuous(&self) {
        let model = self.model.upgrade().unwrap();

        model
            .new_ready(
                None,
                &HashMap::new(),
                Some(Box::new(|mut outputs| {
                    let trigger = outputs.get("trigger");

                    eprintln!("Invoking ready here!");

                    vec![Box::new(Box::pin(Self::ready(trigger)))]
                })),
            )
            .await;
    }

    async fn ready(trigger: Box<dyn Output>) -> ResultStatus {
        let _ = trigger.send_one(().into()).await;
        trigger.close().await;
        ResultStatus::Ok
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}
}
