//! Debug events only carry parameter values at detailed level.

use async_std::channel::unbounded;
use melodium::{load_raw, LoadingConfig};
use melodium_common::executive::{Level, Value};
use melodium_engine::debug::{DebugLevel, Event, EventKind};
use std::{collections::HashMap, sync::Arc};

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = debug_parameters
#! version = 0.11.0
#! require = std:0.11.0 http:0.11.0

use std/engine/util::startup
use std/flow::emit
use std/engine/log::logInfo
use std/data/string_map::|map
use std/ops/option::|wrap
use http/client::HttpClient

treatment main(const token: string)
  model client: HttpClient(base_url = |wrap<string>(token), headers = |map([]))
{
    startup()
    greet(text = token)

    startup.trigger -> greet.trigger
}

treatment greet(var text: string)
  input trigger: Block<void>
{
    emitText: emit<string>(value=text)
    logInfo(label="greet")

    Self.trigger -> emitText.trigger,emit -> logInfo.message
}
"#;

const TOKEN: &str = "debug-parameter-token-sentinel";

fn events(level: DebugLevel) -> Vec<Event> {
    let (pkg, collection) = load_raw(
        Arc::new(SCRIPT.as_bytes().to_vec()),
        "main",
        LoadingConfig {
            core_packages: Vec::new(),
            search_locations: Vec::new(),
            raw_elements: Vec::new(),
        },
    )
    .into_result()
    .expect("script loads");
    let entrypoint = pkg.entrypoints().get("main").cloned().unwrap();

    let engine = melodium_engine::new_engine(collection, Level::Info, level);
    let (debug_sender, debug_receiver) = unbounded();
    engine.add_debug_listener(debug_sender);
    assert!(engine
        .genesis(
            &entrypoint,
            HashMap::from([("token".to_string(), Value::String(TOKEN.to_string()))]),
        )
        .is_success());
    async_std::task::block_on(async {
        engine.live().await;
        engine.end().await;
    });

    let mut events = Vec::new();
    while let Ok(event) = debug_receiver.try_recv() {
        events.push(event);
    }
    events
}

fn built_parameters(events: &[Event]) -> Vec<HashMap<String, Value>> {
    events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::ModelBuilt { parameters, .. } => Some(parameters.clone()),
            EventKind::TreatmentBuilt { environment, .. } => Some(environment.variables().clone()),
            _ => None,
        })
        .filter(|parameters| !parameters.is_empty())
        .collect()
}

#[test]
fn basic_debug_events_give_parameter_names_only() {
    let events = events(DebugLevel::Basic);

    let parameters = built_parameters(&events);
    assert!(parameters
        .iter()
        .any(|parameters| parameters.contains_key("base_url")));
    assert!(parameters
        .iter()
        .any(|parameters| parameters.contains_key("text")));
    for parameters in &parameters {
        for value in parameters.values() {
            assert_eq!(value, &Value::Void(()));
        }
    }

    // Nothing written out contains the value.
    for event in &events {
        let json = serde_json::to_string(&melodium_share::Event::from(event)).unwrap();
        assert!(!json.contains(TOKEN), "{}", json);
    }
}

#[test]
fn detailed_debug_events_give_parameter_values() {
    let events = events(DebugLevel::Detailed);

    let parameters = built_parameters(&events);
    assert!(parameters
        .iter()
        .any(|parameters| parameters.get("base_url")
            == Some(&Value::Option(Some(Box::new(Value::String(
                TOKEN.to_string()
            )))))));
    assert!(parameters
        .iter()
        .any(|parameters| parameters.get("text") == Some(&Value::String(TOKEN.to_string()))));
}
