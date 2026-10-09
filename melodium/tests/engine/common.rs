//! Running programs in the engine, and reading what it gives.

use async_std::channel::unbounded;
use melodium::{load_raw, LoadingConfig};
use melodium_common::descriptor::{Collection, Identifier};
use melodium_common::executive::{Level, Log, Value};
use melodium_engine::debug::{DataContent, DebugLevel, Event, EventKind};
use std::{collections::HashMap, sync::Arc};

/// Loads `script`, giving its collection and its `main` entrypoint.
pub fn load(script: &str) -> (Arc<Collection>, Identifier) {
    let (pkg, collection) = load_raw(
        Arc::new(script.as_bytes().to_vec()),
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
    (collection, entrypoint)
}

/// Runs the `main` entrypoint of `script` with `parameters` until it ends, giving its logs
/// and its debug events, at the detailed level so that sent data is in them.
pub fn run(script: &str, parameters: HashMap<String, Value>) -> (Vec<Log>, Vec<Event>) {
    let (collection, entrypoint) = load(script);
    let engine = melodium_engine::new_engine(collection, Level::Info, DebugLevel::Detailed);
    let (logs_sender, logs_receiver) = unbounded();
    let (debug_sender, debug_receiver) = unbounded();
    engine.add_logs_listener(logs_sender);
    engine.add_debug_listener(debug_sender);

    assert!(engine.genesis(&entrypoint, parameters).is_success());
    async_std::task::block_on(async {
        engine.live().await;
        engine.end().await;
    });

    let mut logs = Vec::new();
    while let Ok(log) = logs_receiver.try_recv() {
        logs.push(log);
    }
    let mut events = Vec::new();
    while let Ok(event) = debug_receiver.try_recv() {
        events.push(event);
    }
    (logs, events)
}

/// Errors of the genesis of `script`, which has to fail.
pub fn genesis_errors(script: &str) -> Vec<String> {
    let (collection, entrypoint) = load(script);
    let engine = melodium_engine::new_engine(collection, Level::Info, DebugLevel::None);
    let result = engine.genesis(&entrypoint, HashMap::new());
    assert!(result.is_failure());
    result
        .failure()
        .into_iter()
        .chain(result.errors().iter())
        .map(|error| error.to_string())
        .collect()
}

/// Values sent by the output `output` of the instance labelled `label`.
pub fn sent(events: &[Event], label: &str, output: &str) -> Vec<Value> {
    events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::DataSent {
                output: sent_by,
                data: DataContent::Values { values },
                ..
            } if sent_by.label == label && sent_by.name == output => Some(values.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// Values sent by the `value` output of the instance labelled `label`, as `reveal` gives.
pub fn revealed(events: &[Event], label: &str) -> Vec<Value> {
    sent(events, label, "value")
}

/// The first log labelled `label`, if any.
pub fn log<'a>(logs: &'a [Log], label: &str) -> Option<&'a Log> {
    logs.iter().find(|log| log.label == label)
}

/// Message of the first log labelled `label`.
pub fn message<'a>(logs: &'a [Log], label: &str) -> &'a str {
    &log(logs, label)
        .unwrap_or_else(|| panic!("no '{}' log in {:?}", label, logs))
        .message
}
