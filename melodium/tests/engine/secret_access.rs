//! Secrets given as parameters are resolved when revealed, every reveal is recorded
//! as a debug event with the identity of the revealing element, refused and failed
//! ones are also logged, and failures surface as errors of the revealing element.
//! Secrets from the environment and from files are never plainly revealed.
//! Revealed values are masked in logs, so they are checked through the data
//! sent by `reveal`, captured in detailed debug events.

use super::common::{self, log, revealed};
use melodium_common::descriptor::{Collection, DataType, Identifier};
use melodium_common::executive::{Level, Log, Secret, Value};
use melodium_engine::debug::{DebugLevel, Event, EventKind};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

const SCRIPT: &str = include_str!("scripts/secret_access.mel");

fn temp_file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_secret_access_{}_{name}",
        std::process::id()
    ));
    std::fs::write(&path, content).unwrap();
    path
}

fn load() -> (Arc<Collection>, Identifier) {
    common::load(SCRIPT)
}

fn run(params: HashMap<String, Value>) -> (Vec<Log>, Vec<Event>) {
    common::run(SCRIPT, params)
}

#[test]
fn plain_reveal_is_denied_by_default_and_logged() {
    let token = temp_file("denied_token", "unused");
    let (logs, events) = run(HashMap::from([(
        "token".to_string(),
        Value::String(format!("file:{}", token.display())),
    )]));

    let denial = log(&logs, "secret").expect("denial logged");
    assert_eq!(denial.level, Level::Error);
    assert!(
        denial.message.contains("MELODIUM_SECRET_ACCESS_TEST_UNSET"),
        "{}",
        denial.message
    );
    assert!(denial
        .message
        .contains("denied to std/secret::reveal (as 'revealPassword')"));

    let error = log(&logs, "password-error").expect("element error");
    assert!(
        error.message.contains("does not allow plain reveal"),
        "{}",
        error.message
    );
    assert!(log(&logs, "password").is_none());

    assert!(events.iter().any(|event| matches!(
        &event.kind,
        EventKind::SecretDenied { element, label: Some(label), track_id: Some(_), .. }
            if element.to_string() == "std/secret::reveal" && label == "revealPassword"
    )));
}

#[test]
fn reveals_are_recorded_with_accessor_identity() {
    let (logs, events) = run(HashMap::from([(
        "token".to_string(),
        Value::String("missing:token".to_string()),
    )]));

    assert_eq!(
        revealed(&events, "revealConcealed"),
        vec![Value::String("concealed-sentinel".to_string())]
    );
    // Once revealed, the value is masked in logs.
    assert_eq!(
        log(&logs, "concealed")
            .expect("concealed value logged")
            .message,
        "<secret \"concealed\">"
    );
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        EventKind::SecretRevealed { secret_name, element, label: Some(label), track_id: Some(_), .. }
            if secret_name == "concealed" && element.to_string() == "std/secret::reveal" && label == "revealConcealed"
    )));
}

#[test]
fn environment_and_file_secrets_are_never_plainly_revealed() {
    std::env::set_var("MELODIUM_SECRET_ACCESS_TEST_TOKEN", "environment-sentinel");
    let token = temp_file("plain_token", "file-sentinel");
    for (locator, scheme) in [
        ("env:MELODIUM_SECRET_ACCESS_TEST_TOKEN".to_string(), "env"),
        (format!("file:{}", token.display()), "file"),
    ] {
        let (logs, events) = run(HashMap::from([(
            "token".to_string(),
            Value::String(locator),
        )]));

        let error = log(&logs, "token-error").expect("element error");
        assert!(
            error.message.contains(&format!(
                "secrets from '{scheme}:' cannot be plainly revealed"
            )),
            "{}",
            error.message
        );
        assert!(log(&logs, "token").is_none());
        assert!(logs
            .iter()
            .all(|log| !log.message.contains("sentinel") || log.label == "concealed"));
        assert!(events.iter().any(|event| matches!(
            &event.kind,
            EventKind::SecretDenied { element, label: Some(label), .. }
                if element.to_string() == "std/secret::reveal" && label == "revealToken"
        )));
    }
}

#[test]
fn missing_sources_are_errors_of_the_revealing_element() {
    let (logs, events) = run(HashMap::from([(
        "token".to_string(),
        Value::String("missing:token".to_string()),
    )]));

    let error = log(&logs, "token-error").expect("element error");
    assert!(
        error.message.contains("no secret source 'missing'"),
        "{}",
        error.message
    );
    assert!(logs.iter().any(|log| log.label == "secret"
        && log.level == Level::Error
        && log
            .message
            .contains("could not be resolved for std/secret::reveal (as 'revealToken')")));
    assert!(events
        .iter()
        .any(|event| matches!(&event.kind, EventKind::SecretResolveFailed { .. })));
}

#[test]
fn environment_and_file_secrets_are_resolved_when_checked() {
    std::env::set_var("MELODIUM_SECRET_ACCESS_TEST_CHECKED", "checked");
    let file = temp_file("checked_file", "checked");
    let missing = std::env::temp_dir().join(format!(
        "melodium_secret_access_{}_checked_missing",
        std::process::id()
    ));
    let secret = |locator: String| Secret::from_locator(&locator, DataType::String).unwrap();

    let engine = melodium_engine::new_engine(
        Arc::new(melodium_common::descriptor::Collection::new()),
        Level::Info,
        DebugLevel::None,
    );
    async_std::task::block_on(async {
        assert!(engine
            .check_secrets(vec![
                (
                    "variable".to_string(),
                    secret("env:MELODIUM_SECRET_ACCESS_TEST_CHECKED".to_string())
                ),
                (
                    "file".to_string(),
                    secret(format!("file:{}", file.display()))
                ),
            ])
            .await
            .is_success());

        let result = engine
            .check_secrets(vec![
                (
                    "variable".to_string(),
                    secret("env:MELODIUM_SECRET_ACCESS_TEST_UNSET".to_string()),
                ),
                (
                    "file".to_string(),
                    secret(format!("file:{}", missing.display())),
                ),
            ])
            .await;
        let errors = result
            .failure()
            .into_iter()
            .chain(result.errors().iter())
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(errors.contains("is not set"), "{}", errors);
        assert!(errors.contains("cannot be read"), "{}", errors);

        // Variables not meant for programs are not given, even when set.
        std::env::set_var("SECRET_ACCESS_TEST_UNPREFIXED", "unprefixed");
        let result = engine
            .check_secrets(vec![(
                "variable".to_string(),
                secret("env:SECRET_ACCESS_TEST_UNPREFIXED".to_string()),
            )])
            .await;
        let error = result.failure().expect("refused").to_string();
        assert!(
            error.contains("only the ones starting with 'MELODIUM_SECRET_' are"),
            "{}",
            error
        );
    });
}

#[test]
fn locator_literals_build_secrets_with_default_policy() {
    let secret = Secret::from_locator("env:NAME", DataType::String).unwrap();
    assert_eq!(secret.name(), "env:NAME");
    assert_eq!(secret.locator(), Some("env:NAME"));
    assert!(!secret.policy().plain_reveal);
}

const STD_SECRET_SCRIPT: &str = include_str!("scripts/std_secret.mel");

#[test]
fn std_secret_conceals_names_and_narrows() {
    let (logs, events) = common::run(STD_SECRET_SCRIPT, HashMap::new());

    assert_eq!(
        revealed(&events, "revealValue"),
        vec![Value::String("concealed-sentinel".to_string())]
    );
    assert_eq!(
        log(&logs, "concealed").expect("logged").message,
        "<secret \"runtime\">"
    );
    assert!(log(&logs, "conceal-error")
        .expect("conceal error")
        .message
        .contains("'everywhere' is not a secret transmission"));
    assert!(log(&logs, "narrowed-error")
        .expect("narrowed error")
        .message
        .contains("does not allow std/secret::reveal to reveal it"));
    assert_eq!(log(&logs, "name").expect("name").message, "file-secret");
}

#[test]
fn secrets_only_this_engine_resolves_need_a_source_at_launch() {
    let (collection, entrypoint) = load();
    let engine = melodium_engine::new_engine(collection, Level::Info, DebugLevel::None);
    let result = engine.genesis(
        &entrypoint,
        HashMap::from([
            (
                "password".to_string(),
                Value::Secret(Secret::from_locator("nosuch:password", DataType::String).unwrap()),
            ),
            (
                "token".to_string(),
                Value::String("nosuch:token".to_string()),
            ),
        ]),
    );
    // The password is local, the token sent by reference may be resolved elsewhere.
    let errors = result
        .failure()
        .into_iter()
        .chain(result.errors().iter())
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        errors,
        vec!["D0253: Secret 'nosuch:password' is located in 'nosuch:', for which no source is registered on this engine".to_string()]
    );
}

#[test]
fn only_declaring_elements_get_secrets_access() {
    let (collection, _) = load();
    let descriptor = |identifier: &str| match collection.get(
        &identifier
            .parse::<melodium_common::descriptor::Identifier>()
            .unwrap()
            .into(),
    ) {
        Some(melodium_common::descriptor::Entry::Treatment(treatment)) => treatment.clone(),
        _ => panic!("no treatment {}", identifier),
    };
    assert!(descriptor("std/secret::reveal").secrets_access());
    assert!(!descriptor("std/secret::conceal").secrets_access());
    assert!(!descriptor("std/flow::emit").secrets_access());
}
