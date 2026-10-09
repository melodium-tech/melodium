//! Secrets received at runtime are derived and narrowed by the treatments of
//! `std/secret/block`, as the functions of `std/secret` do for parameters.
//! Revealed values are read from the data sent by `reveal`, captured in detailed
//! debug events.

use super::common::{self, message, revealed};
use melodium_common::executive::{Log, PackedArray, Value};
use melodium_engine::debug::{Event, EventKind};
use std::collections::HashMap;

const SCRIPT: &str = include_str!("scripts/secret_runtime_derivation.mel");

fn run() -> (Vec<Log>, Vec<Event>) {
    common::run(SCRIPT, HashMap::new())
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
