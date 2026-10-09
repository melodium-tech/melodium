//! `vault:` secrets are read from a Vault server when revealed, the `Vault` model
//! logging in with a JWT such as a GitLab CI ID token, and caching what it reads.
//! The server is a mock answering the few requests involved. A `SqlPool` password
//! read from Vault reaches a mock Postgres server asking for it.

use super::common::{self, genesis_errors, sent};
use melodium_common::descriptor::DataType;
use melodium_common::executive::{Log, Secret, SecretPolicy, SecretTransmission, Value};
use melodium_engine::debug::{DataContent, Event, EventKind};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
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

const SCRIPT: &str = include_str!("scripts/secret_vault.mel");

/// Runs `SCRIPT` against the vault at `address`, with the other parameters of its `Vault`.
fn run(address: &str, parameters: Vec<(&str, Value)>) -> (Vec<Log>, Vec<Event>) {
    let mut parameters: HashMap<String, Value> = parameters
        .into_iter()
        .map(|(name, value)| (name.to_string(), value))
        .collect();
    parameters.insert("address".to_string(), Value::String(address.to_string()));
    run_script(SCRIPT.to_string(), parameters)
}

fn text(value: &str) -> Value {
    Value::String(value.to_string())
}

fn run_script(script: String, parameters: HashMap<String, Value>) -> (Vec<Log>, Vec<Event>) {
    std::env::set_var("MELODIUM_SECRET_VAULT_ID_TOKEN", ID_TOKEN);
    std::env::set_var("MELODIUM_SECRET_VAULT_TOKEN", CLIENT_TOKEN);

    common::run(&script, parameters)
}

#[test]
fn vault_secrets_are_read_with_jwt_authentication() {
    let (address, requests) = vault_server(false);
    let (logs, events) = run(&address, vec![("auth", text("jwt")), ("role", text("app"))]);

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

    // `get` gives a secret located in vault, after checking it exists,
    // with the default policy unless given another one.
    match sent(&events, "getUser", "secret").as_slice() {
        [Value::Secret(secret)] => {
            assert_eq!(secret.name(), "db_user");
            assert_eq!(secret.locator(), Some("vault:kv/data/app/db#user"));
            assert_eq!(secret.policy(), &SecretPolicy::default());
        }
        other => panic!("one secret expected, got {:?}", other),
    }
    match sent(&events, "getShown", "secret").as_slice() {
        [Value::Secret(secret)] => {
            assert_eq!(secret.policy().transmission, SecretTransmission::Value);
            assert!(secret.policy().plain_reveal);
        }
        other => panic!("one secret expected, got {:?}", other),
    }
    assert_eq!(
        sent(&events, "revealShown", "value"),
        vec![Value::String("app".to_string())]
    );
    let invalid = logs
        .iter()
        .find(|log| log.label == "invalid")
        .expect("invalid transmission error");
    assert!(
        invalid
            .message
            .contains("'everywhere' is not a secret transmission"),
        "{}",
        invalid.message
    );
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
            if secret_name == "env:MELODIUM_SECRET_VAULT_ID_TOKEN" && element.to_string() == "vault::Vault"
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
    let (_, events) = run(&address, vec![("auth", text("token"))]);

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
        vec![
            ("auth", text("jwt")),
            ("role", text("app")),
            (
                "jwt",
                Value::Secret(
                    Secret::from_locator("vault:kv/data/app/db#password", DataType::String)
                        .unwrap(),
                ),
            ),
        ],
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
    let (_, events) = run(
        &address,
        vec![
            ("auth", text("jwt")),
            ("role", text("app")),
            ("cache_ttl", Value::U64(0)),
        ],
    );

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
    // The `get` treatments run at the same time, without cache: each reads its secret
    // to check it exists, and the revealed one is read again when revealed.
    let mut gets = requests[4..].to_vec();
    gets.sort();
    assert_eq!(
        gets,
        vec![
            "GET /v1/kv/data/app/db".to_string(),
            "GET /v1/kv/data/app/db".to_string(),
            "GET /v1/kv/data/app/db".to_string(),
            "GET /v1/kv/data/app/missing".to_string(),
        ]
    );
}

const SQL_SCRIPT: &str = include_str!("scripts/secret_vault_sql.mel");

/// Starts a mock Postgres server asking for a cleartext password, giving a
/// connection URL without password and the passwords it receives.
fn postgres_server() -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("postgres://app@{}/app", listener.local_addr().unwrap());
    let passwords = Arc::new(Mutex::new(Vec::new()));

    let received = Arc::clone(&passwords);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let _ = postgres_login(&mut stream, &received);
            }
        }
    });

    (url, passwords)
}

/// Reads a message body preceded by its length, the length counting itself.
fn postgres_message(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut length = [0u8; 4];
    stream.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length > 65536 {
        return Err(std::io::Error::other("unexpected message"));
    }
    let mut body = vec![0u8; length.saturating_sub(4)];
    stream.read_exact(&mut body)?;
    Ok(body)
}

fn postgres_login(stream: &mut TcpStream, passwords: &Mutex<Vec<String>>) -> std::io::Result<()> {
    // TLS is refused, then comes the startup message.
    while postgres_message(stream)?.starts_with(&80877103u32.to_be_bytes()) {
        stream.write_all(b"N")?;
    }
    // AuthenticationCleartextPassword
    stream.write_all(&[b'R', 0, 0, 0, 8, 0, 0, 0, 3])?;
    let mut tag = [0u8; 1];
    stream.read_exact(&mut tag)?;
    let body = postgres_message(stream)?;
    if tag[0] == b'p' {
        passwords
            .lock()
            .unwrap()
            .push(String::from_utf8_lossy(body.strip_suffix(&[0]).unwrap_or(&body)).to_string());
    }
    let fields = b"SFATAL\0C28P01\0Mpassword authentication failed\0\0";
    let mut error = vec![b'E'];
    error.extend_from_slice(&(fields.len() as u32 + 4).to_be_bytes());
    error.extend_from_slice(fields);
    stream.write_all(&error)
}

#[test]
fn sql_pool_password_comes_from_vault() {
    let (address, requests) = vault_server(false);
    let (database, passwords) = postgres_server();
    let (logs, events) = run_script(
        SQL_SCRIPT.to_string(),
        HashMap::from([
            ("address".to_string(), Value::String(address)),
            ("database".to_string(), Value::String(database)),
        ]),
    );

    let passwords = passwords.lock().unwrap().clone();
    assert!(!passwords.is_empty(), "{:?}", logs);
    assert!(passwords.iter().all(|password| password == PASSWORD));
    assert!(requests
        .lock()
        .unwrap()
        .contains(&"GET /v1/kv/data/app/db".to_string()));

    assert!(
        logs.iter().any(|log| log.label == "sql-error"),
        "{:?}",
        logs
    );
    assert!(logs.iter().all(|log| !log.message.contains(PASSWORD)));
    let values = event_values(&events);
    assert!(values
        .iter()
        .any(|value| value.contains("name: \"db_password\"")));
    assert!(values.iter().all(|value| !value.contains(PASSWORD)));
}

/// Values held by debug events, as model parameters or transmitted data.
fn event_values(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .flat_map(|event| match &event.kind {
            EventKind::ModelBuilt { parameters, .. } => parameters.values().cloned().collect(),
            EventKind::DataSent {
                data: DataContent::Values { values },
                ..
            }
            | EventKind::DataTransmitted {
                data: DataContent::Values { values },
                ..
            }
            | EventKind::DataReceived {
                data: DataContent::Values { values },
                ..
            } => values.clone(),
            _ => Vec::new(),
        })
        .map(|value| format!("{:?}", value))
        .collect()
}

#[test]
fn sources_are_registered_before_launch_and_once() {
    let duplicate = genesis_errors(include_str!("scripts/secret_vault_duplicate.mel"));
    assert_eq!(
        duplicate,
        vec!["D0252: A secret source is already registered for 'vault:'".to_string()]
    );

    let missing = genesis_errors(include_str!("scripts/secret_vault_missing.mel"));
    assert_eq!(
        missing,
        vec!["D0253: Secret 'db_password' is located in 'vault:', for which no source is registered on this engine".to_string()]
    );
}
