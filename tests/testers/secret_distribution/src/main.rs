//! Secrets cross to a distant engine following their transmission policy: by reference
//! the distant engine resolves the locator with its own sources, by value the resolved
//! value is sent over TLS only, and local secrets are refused with an explicit error.
//!
//! Each test runs an orchestrating `melodium run` and, when a connection is needed,
//! a `melodium dist --localhost` node, both with different values for the same
//! variables. The node reveals what it receives and sends it back, and values known
//! to the orchestrator are masked in its logs. Secrets held inside data values, such as
//! process environments, follow the same rules. Received references stay secrets from
//! the environment of the node, which it never reveals plainly: its refusals come back
//! with its debug events, and the commands it runs get the values.

#![cfg_attr(not(unix), allow(dead_code, unused_imports))]

use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

const SCRIPT: &str = include_str!("../../../secret_distribution.mel");

/// Sends a process environment with secrets to a node, that runs a command with it.
#[cfg(unix)]
const ENVIRONMENT_SCRIPT: &str = include_str!("../../../secret_environment.mel");

const GROUP_ID: &str = "10101010-1010-1010-1010-101010101010";
const NODE_RECV_KEY: &str = "11111111-1111-1111-1111-111111111111";
const NODE_SEND_KEY: &str = "22222222-2222-2222-2222-222222222222";

fn script(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_secret_distribution_{}_{name}.mel",
        std::process::id()
    ));
    std::fs::write(&path, content).unwrap();
    path
}

fn probe() -> PathBuf {
    script("probe", SCRIPT)
}

/// Starts a node, that ends a few seconds after the orchestrator connects
/// and sends its debug events to it.
fn node(port: u16, tls: bool) -> Child {
    let mut command = Command::new("melodium");
    command
        .args(["dist", "--localhost", "--port", &port.to_string()])
        .args(["--debug-level", "basic"])
        .args(["--wait", "30", "--duration", "5"])
        .env("MELODIUM_GROUP_ID", GROUP_ID)
        .env("MELODIUM_DIST_RECV_KEY", NODE_RECV_KEY)
        .env("MELODIUM_DIST_SEND_KEY", NODE_SEND_KEY)
        .env("MELODIUM_SECRET_DIST_PARAM_TOKEN", "node-param-value")
        .env("MELODIUM_SECRET_DIST_DATA_TOKEN", "node-data-value")
        .env("MELODIUM_SECRET_DIST_ENV_TOKEN", "node-token")
        .env("MELODIUM_SECRET_DIST_ENV_INPUT", "node-input")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if !tls {
        command.arg("--disable-tls");
    }
    let child = command.spawn().unwrap();
    std::thread::sleep(Duration::from_secs(3));
    child
}

fn orchestrate(script: &PathBuf, port: u16, options: &[&str], args: &[&str]) -> String {
    let output: Output = Command::new("melodium")
        .arg("run")
        .args(options)
        .arg(script)
        .args(["--port", &port.to_string()])
        .args(["--send_key", NODE_RECV_KEY, "--recv_key", NODE_SEND_KEY])
        .args(args)
        .env("MELODIUM_GROUP_ID", GROUP_ID)
        .env(
            "MELODIUM_SECRET_DIST_PARAM_TOKEN",
            "orchestrator-param-value",
        )
        .env("MELODIUM_SECRET_DIST_DATA_TOKEN", "orchestrator-data-value")
        .env("MELODIUM_SECRET_DIST_ENV_TOKEN", "orchestrator-token")
        .env("MELODIUM_SECRET_DIST_ENV_INPUT", "orchestrator-input")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn with_node(script: &PathBuf, port: u16, tls: bool, options: &[&str], args: &[&str]) -> String {
    let mut node = node(port, tls);
    let stdout = orchestrate(script, port, options, args);
    let _ = node.kill();
    let _ = node.wait();
    stdout
}

/// Debug file of the orchestrator for `test`.
fn debug_file(test: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "melodium_secret_distribution_{}_{test}_debug.json",
        std::process::id()
    ))
}

/// Secret names and reasons of the plain reveals the node refused, from the debug
/// events it sent to the orchestrator.
fn distant_denials(debug: &PathBuf) -> Vec<(String, String)> {
    let debug = std::fs::read_to_string(debug).unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_str(&debug).unwrap();
    events
        .iter()
        .filter_map(|event| event["kind"]["distant"]["text"].as_str())
        .map(|text| serde_json::from_str::<serde_json::Value>(text).unwrap())
        .filter_map(|event| {
            event["kind"].get("secret_denied").map(|denied| {
                (
                    denied["secret_name"].as_str().unwrap().to_string(),
                    denied["reason"].as_str().unwrap().to_string(),
                )
            })
        })
        .collect()
}

fn assert_refused_by_node(debug: &PathBuf, names: &[&str]) {
    let denials = distant_denials(debug);
    for name in names {
        assert!(
            denials.iter().any(|(secret, reason)| secret == name
                && reason == "secrets from 'env:' cannot be plainly revealed"),
            "no refusal of {:?} in {:?}",
            name,
            denials
        );
    }
}

fn assert_contains(stdout: &str, expected: &str) {
    assert!(
        stdout.contains(expected),
        "expected {:?} in:\n{}",
        expected,
        stdout
    );
}

fn references_reach_the_distant_engine_as_locators() {
    let debug = debug_file("references");
    let stdout = with_node(
        &probe(),
        62611,
        true,
        &["--debug", debug.to_str().unwrap()],
        &[],
    );
    // Secrets from the environment of the node are not plainly revealed there.
    assert_refused_by_node(&debug, &["param_token", "data_token"]);
    assert!(!stdout.contains("param:"), "{}", stdout);
    assert!(!stdout.contains("revealed:"), "{}", stdout);
    assert!(!stdout.contains("node-"), "{}", stdout);
    assert!(!stdout.contains("orchestrator-"), "{}", stdout);
    let debug = std::fs::read_to_string(&debug).unwrap();
    assert!(!debug.contains("node-"), "{}", debug);
    assert!(!debug.contains("orchestrator-"), "{}", debug);
}

fn values_are_resolved_by_the_orchestrating_engine() {
    let debug = std::env::temp_dir().join(format!(
        "melodium_secret_distribution_{}_debug.json",
        std::process::id()
    ));
    let stdout = with_node(
        &probe(),
        62612,
        true,
        &["--debug", debug.to_str().unwrap()],
        &["--param", "value", "--data", "value"],
    );
    // Values the orchestrator resolved are masked in its logs.
    assert_contains(&stdout, "param: <secret \"param_token\">");
    assert_contains(&stdout, "revealed: <secret \"data_token\">");
    assert!(!stdout.contains("node-"), "{}", stdout);
    assert!(!stdout.contains("orchestrator-"), "{}", stdout);

    // Transmissions are recorded by the orchestrator, reveals by the node and sent back.
    let debug = std::fs::read_to_string(&debug).unwrap();
    assert!(!debug.contains("orchestrator-"), "{}", debug);
    let events: Vec<serde_json::Value> = serde_json::from_str(&debug).unwrap();
    let transmitted: Vec<_> = events
        .iter()
        .filter_map(|event| event["kind"].get("secret_transmitted"))
        .map(|event| {
            (
                event["secret_name"].as_str().unwrap(),
                event["transmission"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        transmitted,
        vec![("param_token", "value"), ("data_token", "value")]
    );
    let revealed: Vec<serde_json::Value> = events
        .iter()
        .filter_map(|event| event["kind"]["distant"]["text"].as_str())
        .map(|text| serde_json::from_str(text).unwrap())
        .filter(|event: &serde_json::Value| event["kind"].get("secret_revealed").is_some())
        .collect();
    assert_eq!(revealed.len(), 2, "{:?}", revealed);
}

fn local_secrets_are_refused() {
    // Parameters are refused before connecting.
    let stdout = orchestrate(&probe(), 62613, &[], &["--param", "local"]);
    assert_contains(
        &stdout,
        "start: Cannot distribute, parameter 'token' cannot be sent to the distant engine, access denied: its policy keeps it on this engine",
    );

    let debug = debug_file("local");
    let stdout = with_node(
        &probe(),
        62614,
        true,
        &["--debug", debug.to_str().unwrap()],
        &["--data", "local"],
    );
    assert_contains(
        &stdout,
        "distrib: Cannot send 'data' to the distant engine, access denied: its policy keeps it on this engine",
    );
    // The parameter, sent by reference, reached the node.
    assert_refused_by_node(&debug, &["param_token"]);
    assert!(!stdout.contains("revealed:"), "{}", stdout);
}

fn values_are_refused_over_plain_tcp() {
    let stdout = orchestrate(
        &probe(),
        62615,
        &[],
        &["--plain", "true", "--param", "value"],
    );
    assert_contains(
        &stdout,
        "start: Cannot distribute, parameter 'token' cannot be sent to the distant engine, access denied: its value can only be sent over an encrypted connection",
    );

    let debug = debug_file("plain");
    let stdout = with_node(
        &probe(),
        62616,
        false,
        &["--debug", debug.to_str().unwrap()],
        &["--plain", "true", "--data", "value"],
    );
    assert_contains(
        &stdout,
        "distrib: Cannot send 'data' to the distant engine, access denied: its value can only be sent over an encrypted connection",
    );
    // The parameter, sent by reference, reached the node.
    assert_refused_by_node(&debug, &["param_token"]);
    assert!(!stdout.contains("orchestrator-"), "{}", stdout);
}

#[cfg(unix)]
fn secrets_inside_data_values_follow_their_policy() {
    let script = script("environment", ENVIRONMENT_SCRIPT);
    for (port, transmission, expected) in [
        (
            62617,
            "value",
            Some(("orchestrator-token", "orchestrator-input")),
        ),
        (62618, "reference", Some(("node-token", "node-input"))),
        (62619, "local", None),
    ] {
        let directory = std::env::temp_dir().join(format!(
            "melodium_secret_distribution_{}_{transmission}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();

        let stdout = with_node(
            &script,
            port,
            true,
            &[],
            &[
                "--directory",
                directory.to_str().unwrap(),
                "--transmission",
                transmission,
            ],
        );
        let written = (
            std::fs::read_to_string(directory.join("variable")).ok(),
            std::fs::read_to_string(directory.join("input")).ok(),
        );
        match expected {
            Some((variable, input)) => {
                assert_eq!(
                    written,
                    (Some(variable.to_string()), Some(input.to_string())),
                    "{}",
                    stdout
                );
            }
            None => {
                assert_eq!(written, (None, None), "{}", stdout);
                assert_contains(
                    &stdout,
                    "distrib: Cannot send 'environment' to the distant engine, access denied: its policy keeps it on this engine",
                );
            }
        }
    }
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "references_reach_the_distant_engine_as_locators",
        references_reach_the_distant_engine_as_locators,
    ));
    cases.push((
        "values_are_resolved_by_the_orchestrating_engine",
        values_are_resolved_by_the_orchestrating_engine,
    ));
    cases.push(("local_secrets_are_refused", local_secrets_are_refused));
    cases.push((
        "values_are_refused_over_plain_tcp",
        values_are_refused_over_plain_tcp,
    ));
    // The program runs `sh`.
    #[cfg(unix)]
    cases.push((
        "secrets_inside_data_values_follow_their_policy",
        secrets_inside_data_values_follow_their_policy,
    ));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
