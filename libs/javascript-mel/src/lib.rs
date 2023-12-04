#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod engine;

use async_std::sync::RwLock;
use engine::Engine;
use melodium_core::*;
use melodium_macro::{check, mel_model, mel_package, mel_treatment};
use serde_json::Value;
use std::sync::Weak;

/// Provides JavaScript execution engine.
///
/// The JavaScript/ECMAScript engine manages execution of JS language within Mélodium.
/// First, an engine is instancied with `code` parameter, that code can contains functions definitions, variables setup, and whatever seems useful for incoming use.
/// Then `process` treatment is used for doing JavaScript processing.
///
/// Other parameters are defined:
/// - `stack_size_limit`: Maximum stack size the JavaScript code may use, in bytes.
/// - `recursion_limit`: Maximum recursion that can be reached.
/// - `loop_iteration_limit`: Maximum iteration that can occur on any loop.
/// - `strict`: Defines JavaScript interpretation strictness, can be override by `"use strict"` instruction in JavaScript code.
#[derive(Debug)]
#[mel_model(
    param stack_size_limit u64 1024
    param recursion_limit u64 400
    param loop_iteration_limit u64 4294967295
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

/// Executes JavaScript code on values.
///
/// For every incoming `value`, `code` is executed as-is within `engine`.
/// Inside the `code` part the incoming value is reffered as globally-accessible `value` variable.
/// `value` **must** be valid JSON data, in order to be turned into proper JS object.
/// `code` can return any JS object convertible into JSON data.
///
/// If `value` is not proper JSON data, `code` not actually processable JavaScript code, or its return value not convertible into JSON, an empty `result` and `is_valid` `false` value are send.
/// In all other cases, `result` contains JSON string and `is_valid` is `true`.
#[mel_treatment(
    default code ""
    model engine JavaScriptEngine
    input {compo(type=json) hey()} value Stream<string>
    output result Stream<string>
    output is_valid Stream<bool>
)]
pub async fn process(#[mel(compo(type=javascript))] code: string) {
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
