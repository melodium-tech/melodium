//! Secrets received at runtime are derived and narrowed by the treatments of
//! `std/secret/block`, as the functions of `std/secret` do for parameters.
//! Revealed values are read from the data sent by `reveal`, captured in detailed
//! debug events.

use async_std::channel::unbounded;
use melodium::{load_raw, LoadingConfig};
use melodium_common::executive::{Level, Log, PackedArray, Value};
use melodium_engine::debug::{DataContent, DebugLevel, Event, EventKind};
use std::{collections::HashMap, sync::Arc};

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = secret_runtime_derivation
#! version = 0.11.0
#! require = std:0.11.0

use std/engine/util::startup
use std/engine/log::logError
use std/flow::emit
use std/data/map/block::entry
use std/secret::conceal
use std/secret::reveal
use std/secret/block::format
use std/secret/block::base64
use std/secret/block::urlEncode
use std/secret/block::toBytes
use std/secret/block::revealOnlyBy
use std/secret/block::forbidPlainReveal

treatment main()
{
    startup()

    // A token only known at runtime, such as one given by an API response.
    emitToken: emit<string>(value="to:k/en@1")
    concealToken: conceal<string>(name="token", plain_reveal=true)
    startup.trigger -> emitToken.trigger,emit -> concealToken.value

    tokenEntry: entry<Secret<string>>(key="token")
    bearer: format(template="Bearer {token}", name="authorization")
    revealBearer: reveal<string>()
    concealToken.secret -> tokenEntry.value,map -> bearer.entries,secret -> revealBearer.secret

    encode: base64(name="encoded")
    revealEncoded: reveal<string>()
    concealToken.secret -> encode.secret,encoded -> revealEncoded.secret

    urlEncode(name="url_token")
    revealUrl: reveal<string>()
    concealToken.secret -> urlEncode.secret,encoded -> revealUrl.secret

    toBytes(name="token_bytes")
    revealBytes: reveal<Vec<byte>>()
    concealToken.secret -> toBytes.secret,bytes -> revealBytes.secret

    narrow: revealOnlyBy<string>(elements=["other::Element"])
    revealNarrowed: reveal<string>()
    logNarrowed: logError(label="narrowed")
    concealToken.secret -> narrow.secret,narrowed -> revealNarrowed.secret,error -> logNarrowed.message

    forbid: forbidPlainReveal<string>()
    revealForbidden: reveal<string>()
    logForbidden: logError(label="forbidden")
    concealToken.secret -> forbid.secret,narrowed -> revealForbidden.secret,error -> logForbidden.message
}
"#;

fn run() -> (Vec<Log>, Vec<Event>) {
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

    let engine = melodium_engine::new_engine(collection, Level::Info, DebugLevel::Detailed);
    let (logs_sender, logs_receiver) = unbounded();
    let (debug_sender, debug_receiver) = unbounded();
    engine.add_logs_listener(logs_sender);
    engine.add_debug_listener(debug_sender);

    assert!(engine.genesis(&entrypoint, HashMap::new()).is_success());
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

/// Values sent by the `value` output of the instance labelled `label`.
fn revealed(events: &[Event], label: &str) -> Vec<Value> {
    events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::DataSent {
                output,
                data: DataContent::Values { values },
                ..
            } if output.label == label && output.name == "value" => Some(values.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

fn message<'a>(logs: &'a [Log], label: &str) -> &'a str {
    &logs
        .iter()
        .find(|log| log.label == label)
        .unwrap_or_else(|| panic!("no '{}' log in {:?}", label, logs))
        .message
}

#[test]
fn runtime_secrets_are_derived_by_treatments() {
    let (_, events) = run();

    assert_eq!(
        revealed(&events, "revealBearer"),
        vec![Value::String("Bearer to:k/en@1".to_string())]
    );
    // base64 of "to:k/en@1"
    assert_eq!(
        revealed(&events, "revealEncoded"),
        vec![Value::String("dG86ay9lbkAx".to_string())]
    );
    assert_eq!(
        revealed(&events, "revealUrl"),
        vec![Value::String("to%3Ak%2Fen%401".to_string())]
    );
    match revealed(&events, "revealBytes").as_slice() {
        [Value::Packed(PackedArray::Byte(bytes))] => assert_eq!(bytes.as_slice(), b"to:k/en@1"),
        other => panic!("one bytes value expected, got {:?}", other),
    }

    // Revealing a derived secret also records the secret it comes from.
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        EventKind::SecretRevealed { secret_name, label: Some(label), .. }
            if secret_name == "token" && label == "revealBearer"
    )));
}

#[test]
fn runtime_secrets_are_narrowed_by_treatments() {
    let (logs, _) = run();

    assert!(
        message(&logs, "narrowed").contains("does not allow std/secret::reveal to reveal it"),
        "{}",
        message(&logs, "narrowed")
    );
    assert!(
        message(&logs, "forbidden").contains("does not allow plain reveal"),
        "{}",
        message(&logs, "forbidden")
    );
}
