//! Programs run by Cadence.CI read the secrets stored for their organization through
//! `api:<name>` locators, from the API, with the token of their run. Each one is read once,
//! `--check-secrets` included, and never plainly revealed. Elsewhere, these secrets are
//! refused with the reason, and `api` stays reserved: no model can register it.
//!
//! The API is a mock.

#![cfg_attr(not(unix), allow(dead_code, unused_imports))]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};
use tester::{melodium, temp_file, temp_path};

const SCRIPT: &str = include_str!("../../../api_secrets.mel");
const RESERVED_SCRIPT: &str = include_str!("../../../api_secrets_reserved.mel");

const RUN_ID: &str = "15015015-0150-0150-0150-150150150150";
const RUN_TOKEN: &str = "cad_v_api-secrets-run-token";
const PASSWORD: &str = "api-secrets-password-sentinel";

/// Request received by the mock: method and path, then authorization.
type Received = (String, Option<String>);

/// Starts a mock of the API giving secrets, giving its address and the requests it receives.
fn api_server() -> (String, Arc<Mutex<Vec<Received>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let received = Arc::new(Mutex::new(Vec::new()));

    let requests = Arc::clone(&received);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            let mut authorization = None;
            loop {
                let mut line = String::new();
                let _ = reader.read_line(&mut line);
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                if let Some((name, value)) = line.split_once(':') {
                    if name.eq_ignore_ascii_case("authorization") {
                        authorization = Some(value.trim().to_string());
                    }
                }
            }
            let mut parts = request_line.split_whitespace();
            let target = format!(
                "{} {}",
                parts.next().unwrap_or_default(),
                parts.next().unwrap_or_default()
            );
            let (status, body) =
                if target == format!("GET /execution/run/{RUN_ID}/secret/prod/db-password") {
                    ("200 OK", PASSWORD.to_string())
                } else {
                    let name = target.rsplit("/secret/").next().unwrap_or_default();
                    (
                        "404 Not Found",
                        format!("No secret named `{name}` in this organization."),
                    )
                };
            requests.lock().unwrap().push((target, authorization));
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });

    (address, received)
}

fn directory(name: &str) -> PathBuf {
    let path = temp_path("api_secrets", name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// Runs `script` with `--check-secrets`, configured as on a Cadence.CI worker when an API
/// address is given, and as elsewhere otherwise.
fn run(script: &Path, api: Option<&str>, args: &[&str]) -> Output {
    let mut command: Command = melodium();
    command
        .args(["run", "--check-secrets"])
        .arg(script)
        .args(args)
        .env_remove("MELODIUM_API_URL")
        .env_remove("MELODIUM_API_TOKEN")
        .env_remove("MELODIUM_RUN_ID");
    if let Some(address) = api {
        command
            .env("MELODIUM_API_URL", address)
            .env("MELODIUM_API_TOKEN", RUN_TOKEN)
            .env("MELODIUM_RUN_ID", RUN_ID);
    }
    command.output().unwrap()
}

fn assert_contains(text: &str, expected: &str) {
    assert!(
        text.contains(expected),
        "expected {:?} in:\n{}",
        expected,
        text
    );
}

#[cfg(unix)]
fn secrets_are_read_once_from_the_api() {
    let script = temp_file("api_secrets", "script.mel", SCRIPT);
    let directory = directory("read");
    let (address, requests) = api_server();

    let output = run(
        &script,
        Some(&address),
        &["--directory", directory.to_str().unwrap()],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{}{}", stdout, stderr);

    // The command got the value, read once for the check and the reveal.
    assert_eq!(
        std::fs::read_to_string(directory.join("password")).unwrap(),
        PASSWORD
    );
    assert_eq!(
        *requests.lock().unwrap(),
        vec![(
            format!("GET /execution/run/{RUN_ID}/secret/prod/db-password"),
            Some(format!("Bearer {RUN_TOKEN}"))
        )]
    );

    assert_contains(&stdout, "secrets from 'api:' cannot be plainly revealed");
    for value in [PASSWORD, RUN_TOKEN] {
        assert!(!stdout.contains(value), "{}", stdout);
        assert!(!stderr.contains(value), "{}", stderr);
    }

    let _ = std::fs::remove_dir_all(&directory);
}

fn unknown_names_are_reported_at_startup() {
    let script = temp_file("api_secrets", "script.mel", SCRIPT);
    let directory = directory("unknown");
    let (address, _) = api_server();

    let output = run(
        &script,
        Some(&address),
        &[
            "--directory",
            directory.to_str().unwrap(),
            "--password",
            "api:prod/unknown",
        ],
    );
    assert!(!output.status.success());
    assert_contains(
        &String::from_utf8_lossy(&output.stderr),
        "Secret of parameter 'password' cannot be resolved: resolution failed: Cadence.CI answered 404 Not Found: No secret named `prod/unknown` in this organization.",
    );
    assert!(!directory.join("password").exists());

    let _ = std::fs::remove_dir_all(&directory);
}

fn secrets_are_refused_outside_cadence_ci() {
    let script = temp_file("api_secrets", "script.mel", SCRIPT);
    let directory = directory("outside");

    let output = run(&script, None, &["--directory", directory.to_str().unwrap()]);
    assert!(!output.status.success());
    assert_contains(
        &String::from_utf8_lossy(&output.stderr),
        "Secret of parameter 'password' cannot be resolved: resolution failed: 'api:' secrets are only given to programs run by Cadence.CI, and MELODIUM_API_URL, MELODIUM_API_TOKEN, MELODIUM_RUN_ID are not set",
    );

    let _ = std::fs::remove_dir_all(&directory);
}

fn models_cannot_register_api() {
    let script = temp_file("api_secrets", "reserved.mel", RESERVED_SCRIPT);
    for api in [None, Some("http://127.0.0.1:1")] {
        let output = run(&script, api, &[]);
        assert!(!output.status.success());
        assert_contains(
            &String::from_utf8_lossy(&output.stderr),
            "A secret source is already registered for 'api:'",
        );
    }
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    // The program runs `sh`.
    #[cfg(unix)]
    cases.push((
        "secrets_are_read_once_from_the_api",
        secrets_are_read_once_from_the_api,
    ));
    cases.push((
        "unknown_names_are_reported_at_startup",
        unknown_names_are_reported_at_startup,
    ));
    cases.push((
        "secrets_are_refused_outside_cadence_ci",
        secrets_are_refused_outside_cadence_ci,
    ));
    cases.push(("models_cannot_register_api", models_cannot_register_api));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
