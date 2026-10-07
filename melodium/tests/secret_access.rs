//! Secrets given as parameters are resolved when revealed, every reveal is recorded
//! as a debug event with the identity of the revealing element, refused and failed
//! ones are also logged, and failures surface as errors of the revealing element.
//! Secrets from the environment and from files are never plainly revealed.
//! Revealed values are masked in logs, so they are checked through the data
//! sent by `reveal`, captured in detailed debug events.

use async_std::channel::unbounded;
use melodium::{load_raw, LoadingConfig};
use melodium_common::descriptor::DataType;
use melodium_common::executive::{Level, Log, Secret, Value};
use melodium_engine::debug::{DataContent, DebugLevel, Event, EventKind};
use std::{collections::HashMap, path::PathBuf, process::Command, sync::Arc};

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = secret_access
#! version = 0.10.4
#! require = std:0.10.4

use std/engine/util::startup
use std/flow::emit
use std/engine/log::logInfo
use std/engine/log::logError
use std/ops/option/block::unwrap
use std/secret::conceal
use std/secret::reveal
use std/secret::|locate

treatment main(password: Secret<string> = "env:SECRET_ACCESS_TEST_UNSET", token: string)
{
    startup()

    emitPassword: emit<Secret<string>>(value=password)
    revealPassword: reveal<string>()
    logPassword: logInfo(label="password")
    logPasswordError: logError(label="password-error")

    startup.trigger -> emitPassword.trigger,emit -> revealPassword.secret,value -> logPassword.message
    revealPassword.error -> logPasswordError.message

    emitToken: emit<Option<Secret<string>>>(value=|locate<string>(token, "token", "local", true))
    unwrapToken: unwrap<Secret<string>>()
    revealToken: reveal<string>()
    logToken: logInfo(label="token")
    logTokenError: logError(label="token-error")

    startup.trigger -> emitToken.trigger,emit -> unwrapToken.option,value -> revealToken.secret,value -> logToken.message
    revealToken.error -> logTokenError.message

    emitConcealed: emit<string>(value="concealed-sentinel")
    concealValue: conceal<string>(name="concealed", plain_reveal=true)
    revealConcealed: reveal<string>()
    logConcealed: logInfo(label="concealed")

    startup.trigger -> emitConcealed.trigger,emit -> concealValue.value,secret -> revealConcealed.secret,value -> logConcealed.message
}
"#;

fn temp_file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_secret_access_{}_{name}",
        std::process::id()
    ));
    std::fs::write(&path, content).unwrap();
    path
}

fn run(params: HashMap<String, Value>) -> (Vec<Log>, Vec<Event>) {
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

    assert!(engine.genesis(&entrypoint, params).is_success());
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

fn log<'a>(logs: &'a [Log], label: &str) -> Option<&'a Log> {
    logs.iter().find(|log| log.label == label)
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
        denial.message.contains("SECRET_ACCESS_TEST_UNSET"),
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
    std::env::set_var("SECRET_ACCESS_TEST_TOKEN", "environment-sentinel");
    let token = temp_file("plain_token", "file-sentinel");
    for (locator, scheme) in [
        ("env:SECRET_ACCESS_TEST_TOKEN".to_string(), "env"),
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
    std::env::set_var("SECRET_ACCESS_TEST_CHECKED", "checked");
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
                    secret("env:SECRET_ACCESS_TEST_CHECKED".to_string())
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
                    secret("env:SECRET_ACCESS_TEST_UNSET".to_string()),
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
    });
}

#[test]
fn secret_parameters_take_locators_on_the_command_line() {
    let script = temp_file("script.mel", SCRIPT);
    let password = temp_file("cli_password", "password-sentinel\n");
    let token = temp_file("cli_token", "token-sentinel");
    let missing = std::env::temp_dir().join(format!(
        "melodium_secret_access_{}_cli_missing",
        std::process::id()
    ));

    let output = Command::new(env!("CARGO_BIN_EXE_melodium"))
        .args(["run", "--check-secrets"])
        .arg(&script)
        .arg("--password")
        .arg(format!("file:{}", password.display()))
        .arg("--token")
        .arg(format!("file:{}", token.display()))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{}", stdout);
    assert!(!stdout.contains("token-sentinel"), "{}", stdout);
    assert!(!stdout.contains("password-sentinel"), "{}", stdout);
    assert!(stdout.contains(&format!("secret \"file:{}\" denied", password.display())));
    assert!(stdout.contains("secrets from 'file:' cannot be plainly revealed"));

    let output = Command::new(env!("CARGO_BIN_EXE_melodium"))
        .args(["run", "--check-secrets"])
        .arg(&script)
        .arg("--password")
        .arg(format!("file:{}", missing.display()))
        .arg("--token")
        .arg(format!("file:{}", token.display()))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Secret of parameter 'password' cannot be resolved"),
        "{}",
        stderr
    );

    let output = Command::new(env!("CARGO_BIN_EXE_melodium"))
        .arg(&script)
        .args(["--password", "hunter2", "--token", "env:UNUSED"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("is not a secret locator"));
}

#[test]
fn locator_literals_build_secrets_with_default_policy() {
    let secret = Secret::from_locator("env:NAME", DataType::String).unwrap();
    assert_eq!(secret.name(), "env:NAME");
    assert_eq!(secret.locator(), Some("env:NAME"));
    assert!(!secret.policy().plain_reveal);
}

const STD_SECRET_SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = std_secret
#! version = 0.10.4
#! require = std:0.10.4

use std/engine/util::startup
use std/flow::emit
use std/engine/log::logInfo
use std/engine/log::logError
use std/secret::conceal
use std/secret::reveal
use std/secret::|from_environment
use std/secret::|from_file
use std/secret::|name
use std/secret::|reveal_only_by

treatment main()
{
    startup()

    emitValue: emit<string>(value="concealed-sentinel")
    concealValue: conceal<string>(name="runtime", plain_reveal=true)
    revealValue: reveal<string>()
    logValue: logInfo(label="concealed")

    startup.trigger -> emitValue.trigger,emit -> concealValue.value,secret -> revealValue.secret,value -> logValue.message

    emitBad: emit<string>(value="unused")
    concealBad: conceal<string>(name="bad", transmission="everywhere")
    logBad: logError(label="conceal-error")

    startup.trigger -> emitBad.trigger,emit -> concealBad.value,error -> logBad.message

    emitNarrowed: emit<Secret<string>>(value=|reveal_only_by<string>(|from_environment("SECRET_ACCESS_TEST_UNSET", "narrowed"), ["other::Element"]))
    revealNarrowed: reveal<string>()
    logNarrowed: logError(label="narrowed-error")

    startup.trigger -> emitNarrowed.trigger,emit -> revealNarrowed.secret,error -> logNarrowed.message

    emitName: emit<string>(value=|name<string>(|from_file("/run/secrets/db", "file-secret")))
    logName: logInfo(label="name")

    startup.trigger -> emitName.trigger,emit -> logName.message
}
"#;

#[test]
fn std_secret_conceals_names_and_narrows() {
    let (pkg, collection) = load_raw(
        Arc::new(STD_SECRET_SCRIPT.as_bytes().to_vec()),
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
