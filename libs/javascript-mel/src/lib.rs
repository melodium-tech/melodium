#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(any(
    all(feature = "real", feature = "mock"),
    not(any(feature = "real", feature = "mock"))
))]
compile_error!("One of the two features 'real' or 'mock' must be enabled");

mod engine;

use async_std::sync::Mutex;
use engine::Engine;
use json_mel::*;
use melodium_core::*;
use melodium_macro::{check, mel_model, mel_package, mel_treatment};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

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
    param {content(javascript)} code string ""
    initialize initialize
    shutdown shutdown
)]
pub struct JavaScriptEngine {
    model: Weak<JavaScriptEngineModel>,
    engine: Mutex<Option<Engine>>,
}

impl JavaScriptEngine {
    fn new(model: Weak<JavaScriptEngineModel>) -> Self {
        Self {
            model,
            engine: Mutex::new(None),
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

        async_std::task::block_on(async move {
            *model.inner().engine.lock().await = Some(engine);
        });
    }

    fn shutdown(&self) {
        let model = self.model.upgrade().unwrap();
        async_std::task::block_on(async move {
            if let Some(engine) = model.inner().engine.lock().await.as_ref() {
                engine.stop();
            }
        });
    }

    fn invoke_source(&self, _source: &str, _params: HashMap<String, Value>) {}

    pub async fn process(
        &self,
        value: serde_json::Value,
        code: String,
    ) -> Result<Result<serde_json::Value, String>, ()> {
        if let Some(engine) = self.engine.lock().await.as_ref() {
            engine.process(value, code).await
        } else {
            Err(())
        }
    }

    pub(crate) fn engine(&self) -> &Mutex<Option<Engine>> {
        &self.engine
    }
}

/// Executes JavaScript code on values.
///
/// For every incoming `value`, `code` is executed as-is within `engine`.
/// Inside the `code` part the incoming value is reffered as globally-accessible `value` variable.
/// `code` can return any JS object convertible into JSON data.
///
/// If `code` not actually processable JavaScript code, or its return value not convertible into JSON, a none `result` value is send.
#[mel_treatment(
    model engine JavaScriptEngine
    input value Stream<Json>
    output result Stream<Option<Json>>
)]
pub async fn process(#[mel(content(javascript))] code: string) {
    let engine = JavaScriptEngineModel::into(engine);

    'main: while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<Vec<Value>>::into(values))
    {
        for value in values {
            if let Value::Data(value) = value {
                match value.downcast_arc::<Json>() {
                    Ok(value) => {
                        let processed;
                        if let Some(engine) = engine.inner().engine().lock().await.as_ref() {
                            processed = engine.process(value.0.clone(), code.clone()).await;
                        } else {
                            break;
                        }

                        match processed {
                            Ok(Ok(value)) => {
                                check!(
                                    result
                                        .send_one(
                                            Some(Arc::new(Json(value)) as Arc<dyn Data>).into()
                                        )
                                        .await
                                );
                            }
                            Ok(Err(_err)) => {
                                check!(result.send_one(Option::<Arc<dyn Data>>::None.into()).await);
                            }
                            Err(_) => {
                                break 'main;
                            }
                        }
                    }
                    Err(_) => break 'main,
                }
            }
        }
    }
}

mel_package!();
