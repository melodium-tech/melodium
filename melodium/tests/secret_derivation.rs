//! Secrets derived from other secrets are built without revealing anything,
//! computed when revealed, and get the most restrictive policy of their inputs.
//! A secret derived from the environment or a file is never plainly revealed.

use async_std::channel::unbounded;
use melodium::{load_raw, LoadingConfig};
use melodium_common::descriptor::DataType;
use melodium_common::executive::{
    Level, Log, Secret, SecretOrigin, SecretPolicy, SecretReveal, SecretTransmission, Value,
};
use melodium_engine::debug::{DebugLevel, Event, EventKind};
use std::{collections::HashMap, sync::Arc};

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = secret_derivation
#! version = 0.10.4
#! require = std:0.10.4

use std/engine/util::startup
use std/flow::emit
use std/engine/log::logInfo
use std/engine/log::logError
use std/data/map::|map
use std/data/map::|entry
use std/ops/option::|unwrap_or
use std/secret::reveal
use std/secret::|locate
use std/secret::|from_environment
use std/secret::|format
use std/secret::|url_encode
use std/secret::|base64
use std/secret::|base64_bytes
use std/secret::|to_bytes

treatment main(token: Secret<string>, file_token_locator: string)
{
    startup()
    derivations(
        token = token,
        closed = |from_environment("MELODIUM_SECRET_DERIVATION_CLOSED", "closed"),
        from_file = |unwrap_or<Secret<string>>(|locate<string>(file_token_locator, "file_token", "local", true), |from_environment("MELODIUM_SECRET_DERIVATION_UNUSED", "fallback"))
    )

    startup.trigger -> derivations.trigger
}

treatment derivations(var token: Secret<string>, var closed: Secret<string>, var from_file: Secret<string>)
  input trigger: Block<void>
{
    bearer: show(label="bearer", secret=|format("Bearer {token}", |map([|entry<Secret<string>>("token", token)]), "authorization"))
    repository: show(label="repository", secret=|format("https://gitlab-ci-token:{token}@gitlab.com/group/project.git", |map([|entry<Secret<string>>("token", |url_encode(token, "url_token"))]), "repository"))
    basic: show(label="basic", secret=|base64(|format("{user}:{password}", |map([|entry<string>("user", "ci"), |entry<Secret<string>>("password", token)]), "credentials"), "basic"))
    bytes: show(label="bytes", secret=|base64_bytes(|to_bytes(token, "token_bytes"), "token_bytes_base64"))
    mixed: show(label="mixed", secret=|format("{open}{closed}", |map([|entry<Secret<string>>("open", token), |entry<Secret<string>>("closed", closed)]), "mixed"))
    broken: show(label="broken", secret=|format("{missing}", |map([|entry<Secret<string>>("token", token)]), "broken"))
    fileBearer: show(label="file-bearer", secret=|format("Bearer {token}", |map([|entry<Secret<string>>("token", from_file)]), "file_authorization"))

    Self.trigger -> bearer.trigger
    Self.trigger -> repository.trigger
    Self.trigger -> basic.trigger
    Self.trigger -> bytes.trigger
    Self.trigger -> mixed.trigger
    Self.trigger -> broken.trigger
    Self.trigger -> fileBearer.trigger
}

treatment show(label: string, var secret: Secret<string>)
  input trigger: Block<void>
{
    emitSecret: emit<Secret<string>>(value=secret)
    revealSecret: reveal<string>()
    logValue: logInfo(label=label)
    logFailure: logError(label=label)

    Self.trigger -> emitSecret.trigger,emit -> revealSecret.secret,value -> logValue.message
    revealSecret.error -> logFailure.message
}
"#;

const TOKEN: &str = "to:k/en@1";

fn run() -> (Vec<Log>, Vec<Event>) {
    let token = std::env::temp_dir().join(format!(
        "melodium_secret_derivation_{}_token",
        std::process::id()
    ));
    std::fs::write(&token, TOKEN).unwrap();

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

    let engine = melodium_engine::new_engine(collection, Level::Info, DebugLevel::Basic);
    let (logs_sender, logs_receiver) = unbounded();
    let (debug_sender, debug_receiver) = unbounded();
    engine.add_logs_listener(logs_sender);
    engine.add_debug_listener(debug_sender);

    assert!(engine
        .genesis(
            &entrypoint,
            HashMap::from([
                (
                    "token".to_string(),
                    // Inline, as secrets from files are never plainly revealed.
                    Value::Secret(
                        Secret::new(
                            "token".to_string(),
                            DataType::String,
                            SecretPolicy {
                                transmission: SecretTransmission::Local,
                                reveal: SecretReveal::Any,
                                plain_reveal: true,
                            },
                            SecretOrigin::Inline(Value::String(TOKEN.to_string())),
                        )
                        .unwrap(),
                    ),
                ),
                (
                    "file_token_locator".to_string(),
                    Value::String(format!("file:{}", token.display())),
                ),
            ]),
        )
        .is_success());
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

fn message<'a>(logs: &'a [Log], label: &str) -> &'a str {
    &logs
        .iter()
        .find(|log| log.label == label)
        .unwrap_or_else(|| panic!("no '{}' log in {:?}", label, logs))
        .message
}

#[test]
fn derived_secrets_are_computed_when_revealed() {
    let (logs, events) = run();

    assert_eq!(message(&logs, "bearer"), format!("Bearer {TOKEN}"));
    assert_eq!(
        message(&logs, "repository"),
        "https://gitlab-ci-token:to%3Ak%2Fen%401@gitlab.com/group/project.git"
    );
    // base64 of "ci:to:k/en@1"
    assert_eq!(message(&logs, "basic"), "Y2k6dG86ay9lbkAx");
    // base64 of "to:k/en@1"
    assert_eq!(message(&logs, "bytes"), "dG86ay9lbkAx");

    // Building derived secrets reveals nothing: every reveal comes from `reveal`,
    // and revealing a derived secret also records the secrets it comes from.
    for event in &events {
        if let EventKind::SecretRevealed { element, .. } = &event.kind {
            assert_eq!(element.to_string(), "std/secret::reveal");
        }
    }
    let revealed = |name: &str| {
        events
            .iter()
            .filter(|event| matches!(&event.kind, EventKind::SecretRevealed { secret_name, .. } if secret_name == name))
            .count()
    };
    assert_eq!(revealed("authorization"), 1);
    assert_eq!(revealed("url_token"), 1);
    assert_eq!(revealed("token"), 4);
    assert_eq!(revealed("closed"), 0);
}

#[test]
fn derived_secrets_get_the_most_restrictive_policy() {
    let (logs, events) = run();

    assert!(message(&logs, "mixed").contains("does not allow plain reveal"));
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        EventKind::SecretDenied { secret_name, .. } if secret_name == "mixed"
    )));
}

#[test]
fn derivation_failures_never_show_values() {
    let (logs, _) = run();

    let broken = message(&logs, "broken");
    assert!(
        broken.contains("the template uses an entry that is not given"),
        "{}",
        broken
    );
    for log in &logs {
        if log.label == "broken" || log.label == "secret" {
            assert!(!log.message.contains(TOKEN), "{}", log.message);
        }
    }
}

#[test]
fn secrets_derived_from_files_are_never_plainly_revealed() {
    let (logs, events) = run();

    let refused = message(&logs, "file-bearer");
    assert!(
        refused.contains("secrets from 'file:' cannot be plainly revealed"),
        "{}",
        refused
    );
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        EventKind::SecretDenied { secret_name, .. } if secret_name == "file_authorization"
    )));
    for log in &logs {
        assert!(!log.message.contains(TOKEN) || log.label != "file-bearer");
    }
}
