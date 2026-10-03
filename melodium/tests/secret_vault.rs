//! `vault:` secrets are read from a Vault server when revealed, the `Vault` model
//! logging in with a JWT such as a GitLab CI ID token, and caching what it reads.
//! The server is a mock answering the few requests involved.

use async_std::channel::unbounded;
use melodium::{load_raw, LoadingConfig};
use melodium_common::executive::{Level, Log, Value};
use melodium_engine::debug::{DataContent, DebugLevel, Event, EventKind};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

const ID_TOKEN: &str = "eyJ.id-token-sentinel.sig";
const CLIENT_TOKEN: &str = "hvs.client-token-sentinel";
const PASSWORD: &str = "vault-password-sentinel";

/// Starts a mock Vault server, giving its address and the requests it receives.
///
/// When `revoke_first`, the first token given by a login is refused, as if revoked.
fn vault_server(revoke_first: bool) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));

    let received = Arc::clone(&requests);
    std::thread::spawn(move || {
        let mut logins = 0;
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());

            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut headers = HashMap::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                if let Some((name, value)) = line.split_once(':') {
                    headers.insert(name.trim().to_lowercase(), value.trim().to_string());
                }
            }
            let length = headers
                .get("content-length")
                .and_then(|length| length.parse().ok())
                .unwrap_or(0);
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();

            let mut parts = request_line.split_whitespace();
            let request = format!(
                "{} {}",
                parts.next().unwrap_or_default(),
                parts.next().unwrap_or_default()
            );
            received.lock().unwrap().push(request.clone());

            let token = headers.get("x-vault-token").cloned().unwrap_or_default();
            let valid_token = if revoke_first && logins == 1 {
                "hvs.revoked"
            } else {
                CLIENT_TOKEN
            };
            let (status, response) = match request.as_str() {
                "POST /v1/auth/jwt/login" if body["role"] == "app" && body["jwt"] == ID_TOKEN => {
                    logins += 1;
                    let client_token = if revoke_first && logins == 1 {
                        "hvs.revoked-first"
                    } else {
                        CLIENT_TOKEN
                    };
                    (
                        200,
                        serde_json::json!({"auth": {"client_token": client_token, "lease_duration": 3600}}),
                    )
                }
                "POST /v1/auth/jwt/login" => (
                    400,
                    serde_json::json!({"errors": ["invalid role or token"]}),
                ),
                _ if token != valid_token => {
                    (403, serde_json::json!({"errors": ["permission denied"]}))
                }
                "GET /v1/kv/data/app/db" => (
                    200,
                    serde_json::json!({"data": {
                        "data": {"password": PASSWORD, "user": "app"},
                        "metadata": {"version": 1}
                    }}),
                ),
                _ => (404, serde_json::json!({"errors": []})),
            };
            let response = response.to_string();
            write!(
                stream,
                "HTTP/1.1 {status} Status\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                response.len()
            )
            .unwrap();
        }
    });

    (address, requests)
}

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = secret_vault
#! version = 0.10.4
#! require = std:0.10.4 vault:0.10.4

use std/engine/util::startup
use std/flow::emit
use std/flow::stream
use std/flow::trigger
use std/engine/log::logInfo
use std/engine/log::logError
use std/ops/option/block::unwrap
use std/secret::reveal
use std/secret::|locate
use vault::Vault
use vault::get

treatment main(const address: string)
  model vault: Vault(address = address, MODEL_PARAMETERS)
{
    startup()

    emitPassword: emit<Option<Secret<string>>>(value=|locate<string>("vault:kv/data/app/db#password", "db_password", "local", true))
    unwrapPassword: unwrap<Secret<string>>()
    revealPassword: reveal<string>()
    logPassword: logInfo(label="password")
    logPasswordError: logError(label="password-error")

    startup.trigger -> emitPassword.trigger,emit -> unwrapPassword.option,value -> revealPassword.secret,value -> logPassword.message
    revealPassword.error -> logPasswordError.message

    // Once the password is revealed, from the cache.
    stream<string>()
    trigger<string>()
    revealPassword.value -> stream.block,stream -> trigger.stream

    emitUser: emit<string>(value="kv/data/app/db#user")
    getUser: get[vault=vault](name="db_user")
    trigger.end -> emitUser.trigger,emit -> getUser.path

    emitMissing: emit<string>(value="kv/data/app/missing#password")
    getMissing: get[vault=vault](name="missing")
    logMissing: logError(label="missing")
    trigger.end -> emitMissing.trigger,emit -> getMissing.path,error -> logMissing.message
}
"#;

fn run(address: &str, model_parameters: &str) -> (Vec<Log>, Vec<Event>) {
    std::env::set_var("VAULT_ID_TOKEN", ID_TOKEN);
    std::env::set_var("VAULT_TOKEN", CLIENT_TOKEN);

    let (pkg, collection) = load_raw(
        Arc::new(
            SCRIPT
                .replace("MODEL_PARAMETERS", model_parameters)
                .into_bytes(),
        ),
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

    assert!(engine
        .genesis(
            &entrypoint,
            HashMap::from([("address".to_string(), Value::String(address.to_string()))]),
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

/// Values sent by the output `name` of the instance labelled `label`.
fn sent(events: &[Event], label: &str, name: &str) -> Vec<Value> {
    events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::DataSent {
                output,
                data: DataContent::Values { values },
                ..
            } if output.label == label && output.name == name => Some(values.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn vault_secrets_are_read_with_jwt_authentication() {
    let (address, requests) = vault_server(false);
    let (logs, events) = run(&address, "auth = \"jwt\", role = \"app\"");

    // The password comes from vault, and is masked once revealed.
    assert_eq!(
        sent(&events, "revealPassword", "value"),
        vec![Value::String(PASSWORD.to_string())]
    );
    let logged = logs
        .iter()
        .find(|log| log.label == "password")
        .unwrap_or_else(|| panic!("no password log in {:?}", logs));
    assert_eq!(logged.message, "<secret \"db_password\">");

    // `get` gives a secret located in vault, after checking it exists.
    match sent(&events, "getUser", "secret").as_slice() {
        [Value::Secret(secret)] => {
            assert_eq!(secret.name(), "db_user");
            assert_eq!(secret.locator(), Some("vault:kv/data/app/db#user"));
        }
        other => panic!("one secret expected, got {:?}", other),
    }
    let missing = logs
        .iter()
        .find(|log| log.label == "missing")
        .expect("missing secret error");
    assert!(
        missing.message.contains("status 404"),
        "{}",
        missing.message
    );

    // One login, and one read of the secret thanks to the cache.
    let requests = requests.lock().unwrap().clone();
    assert_eq!(
        requests,
        vec![
            "POST /v1/auth/jwt/login".to_string(),
            "GET /v1/kv/data/app/db".to_string(),
            "GET /v1/kv/data/app/missing".to_string(),
        ]
    );

    // The model reveals its own credential, as `vault::Vault`.
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        EventKind::SecretRevealed { secret_name, element, .. }
            if secret_name == "env:VAULT_ID_TOKEN" && element.to_string() == "vault::Vault"
    )));

    for log in &logs {
        for value in [ID_TOKEN, CLIENT_TOKEN, PASSWORD] {
            assert!(!log.message.contains(value), "{}", log.message);
        }
    }
}

#[test]
fn vault_secrets_are_read_with_a_token() {
    let (address, requests) = vault_server(false);
    let (_, events) = run(&address, "auth = \"token\"");

    assert_eq!(
        sent(&events, "revealPassword", "value"),
        vec![Value::String(PASSWORD.to_string())]
    );
    assert_eq!(
        requests.lock().unwrap().clone(),
        vec![
            "GET /v1/kv/data/app/db".to_string(),
            "GET /v1/kv/data/app/missing".to_string(),
        ]
    );
}

#[test]
fn credentials_cannot_come_from_the_same_vault() {
    let (address, requests) = vault_server(false);
    let (logs, _) = run(
        &address,
        "auth = \"jwt\", role = \"app\", jwt = \"vault:kv/data/app/db#password\"",
    );

    let error = logs
        .iter()
        .find(|log| log.label == "password-error")
        .unwrap_or_else(|| panic!("no password error in {:?}", logs));
    assert!(
        error
            .message
            .contains("cannot come from the vault it authenticates to"),
        "{}",
        error.message
    );
    assert!(requests.lock().unwrap().is_empty());
}

#[test]
fn refused_tokens_are_renewed_once() {
    let (address, requests) = vault_server(true);
    let (_, events) = run(&address, "auth = \"jwt\", role = \"app\", cache_ttl = 0");

    assert_eq!(
        sent(&events, "revealPassword", "value"),
        vec![Value::String(PASSWORD.to_string())]
    );
    let requests = requests.lock().unwrap().clone();
    assert_eq!(
        requests[..4],
        [
            "POST /v1/auth/jwt/login".to_string(),
            "GET /v1/kv/data/app/db".to_string(),
            "POST /v1/auth/jwt/login".to_string(),
            "GET /v1/kv/data/app/db".to_string(),
        ]
    );
    // Both `get` run at the same time, without cache.
    let mut gets = requests[4..].to_vec();
    gets.sort();
    assert_eq!(
        gets,
        vec![
            "GET /v1/kv/data/app/db".to_string(),
            "GET /v1/kv/data/app/missing".to_string(),
        ]
    );
}
