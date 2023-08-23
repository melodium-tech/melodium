#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod engine;

use async_std::sync::RwLock;
use engine::Engine;
use melodium_core::*;
use melodium_macro::{check, mel_model, mel_package, mel_treatment};
use serde_json::Value;
use std::sync::Weak;

#[derive(Debug)]
#[mel_model(
    param stack_size_limit u64 1024
    param recursion_limit u64 400
    param loop_iteration_limit u64 18446744073709551615
    param strict bool false
    param code string ""
    initialize initialize
    shutdown shutdown
)]
pub struct JavaScriptEngine {
    model: Weak<JavaScriptEngineModel>,
    engine: RwLock<Option<Engine>>,
}

impl JavaScriptEngine {
    fn new(model: Weak<JavaScriptEngineModel>) -> Self {
        Self {
            model,
            engine: RwLock::new(None),
        }
    }

    fn initialize(&self) {
        let model = self.model.upgrade().unwrap();

        let engine = Engine::new(
            model.get_stack_size_limit(),
            model.get_recursion_limit(),
            model.get_loop_iteration_limit(),
            model.get_strict(),
            model.get_code(),
        );

        async_std::task::block_on(async {
            *self.engine.write().await = Some(engine);
        });
    }

    fn shutdown(&self) {
        async_std::task::block_on(async {
            if let Some(engine) = self.engine.write().await.as_ref() {
                engine.stop();
            }
        });
    }

    pub(crate) fn engine(&self) -> &RwLock<Option<Engine>> {
        &self.engine
    }
}

#[mel_treatment(
    default code ""
    model engine JavaScriptEngine
    input value Stream<string>
    output result Stream<string>
    output is_valid Stream<bool>
)]
pub async fn process(code: string) {
    let engine = JavaScriptEngineModel::into(engine);

    while let Ok(values) = value.recv_string().await {
        for value in values {
            match serde_json::from_str::<Value>(&value) {
                Ok(value) => {
                    let processed;
                    if let Some(engine) = engine.inner().engine().read().await.as_ref() {
                        processed = engine.process(value, code.clone()).await;
                    } else {
                        break;
                    }

                    match processed {
                        Ok(Ok(value)) => {
                            check!(result.send_one_string(value.to_string()).await);
                            let _ = is_valid.send_one_bool(true).await;
                        }
                        Ok(Err(_err)) => {
                            check!(result.send_one_string("".to_string()).await);
                            let _ = is_valid.send_one_bool(false).await;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                Err(_err) => {
                    check!(result.send_one_string("".to_string()).await);
                    let _ = is_valid.send_one_bool(false).await;
                }
            }
        }
    }
}

mel_package!();
