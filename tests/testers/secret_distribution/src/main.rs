//! Secrets cross to a distant engine following their transmission policy: by reference
//! the distant engine resolves the locator with its own sources, by value the resolved
//! value is sent over TLS only, and local secrets are refused with an explicit error.
//!
//! Each test runs a program with `melodium run`, the engine asking for the distribution,
//! and when a connection is needed a `melodium dist --localhost` node, both with
//! different values for the same variables. The node reveals what it receives and sends
//! it back, and values known to the requesting engine are masked in its logs. Received references stay secrets from
//! the environment of the node, which it never reveals plainly: its refusals come back
//! with its debug events.

use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const SCRIPT: &str = include_str!("../../../secret_distribution.mel");

const GROUP_ID: &str = "10101010-1010-1010-1010-101010101010";
const NODE_RECV_KEY: &str = "11111111-1111-1111-1111-111111111111";
const NODE_SEND_KEY: &str = "22222222-2222-2222-2222-222222222222";

fn script() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_secret_distribution_{}.mel",
        std::process::id()
    ));
    std::fs::write(&path, SCRIPT).unwrap();
    path
}

/// Starts a node, that sends its debug events to the requesting engine, and ends once
/// the program stops the distribution, or at the latest 5 seconds after it started.
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
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if !tls {
        command.arg("--disable-tls");
    }
    let child = command.spawn().unwrap();
    wait_listening(port, tls);
    child
}

/// Waits for the node to listen on `port`.
///
/// A TLS node goes on listening after connections failing their handshake, so connecting
/// tells when it listens. A plain TCP node takes the first connection as the distribution
/// one, so it is given a fixed delay instead.
fn wait_listening(port: u16, tls: bool) {
    if !tls {
        std::thread::sleep(Duration::from_secs(3));
        return;
    }
    let start = Instant::now();
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "node not listening on port {port}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Runs the program, asking the node on `port` for a distribution, and gives its
/// standard output.
fn distribute(port: u16, options: &[&str], args: &[&str]) -> String {
    let output: Output = Command::new("melodium")
        .arg("run")
        .args(options)
        .arg(script())
        .args(["--port", &port.to_string()])
        .args(["--send_key", NODE_RECV_KEY, "--recv_key", NODE_SEND_KEY])
        .args(args)
        .env("MELODIUM_GROUP_ID", GROUP_ID)
        .env("MELODIUM_SECRET_DIST_PARAM_TOKEN", "requester-param-value")
        .env("MELODIUM_SECRET_DIST_DATA_TOKEN", "requester-data-value")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn with_node(port: u16, tls: bool, options: &[&str], args: &[&str]) -> String {
    let mut node = node(port, tls);
    let stdout = distribute(port, options, args);
    let _ = node.kill();
    let _ = node.wait();
    stdout
}

/// Debug file of the requesting engine for `test`.
fn debug_file(test: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "melodium_secret_distribution_{}_{test}_debug.json",
        std::process::id()
    ))
}

/// Secret names and reasons of the plain reveals the node refused, from the debug
/// events it sent to the requesting engine.
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
    let stdout = with_node(62611, true, &["--debug", debug.to_str().unwrap()], &[]);
    // Secrets from the environment of the node are not plainly revealed there.
    assert_refused_by_node(&debug, &["param_token", "data_token"]);
    assert!(!stdout.contains("param:"), "{}", stdout);
    assert!(!stdout.contains("revealed:"), "{}", stdout);
    assert!(!stdout.contains("node-"), "{}", stdout);
    assert!(!stdout.contains("requester-"), "{}", stdout);
    let debug = std::fs::read_to_string(&debug).unwrap();
    assert!(!debug.contains("node-"), "{}", debug);
    assert!(!debug.contains("requester-"), "{}", debug);
}

fn values_are_resolved_by_the_requesting_engine() {
    let debug = std::env::temp_dir().join(format!(
        "melodium_secret_distribution_{}_debug.json",
        std::process::id()
    ));
    let stdout = with_node(
        62612,
        true,
        &["--debug", debug.to_str().unwrap()],
        &["--param", "value", "--data", "value"],
    );
    // Values the requesting engine resolved are masked in its logs.
    assert_contains(&stdout, "param: <secret \"param_token\">");
    assert_contains(&stdout, "revealed: <secret \"data_token\">");
    assert!(!stdout.contains("node-"), "{}", stdout);
    assert!(!stdout.contains("requester-"), "{}", stdout);

    // Transmissions are recorded by the requesting engine, reveals by the node and sent
    // back.
    let debug = std::fs::read_to_string(&debug).unwrap();
    assert!(!debug.contains("requester-"), "{}", debug);
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
    let stdout = distribute(62613, &[], &["--param", "local"]);
    assert_contains(
        &stdout,
        "start: Cannot distribute, parameter 'token' cannot be sent to the distant engine, access denied: its policy keeps it on this engine",
    );

    let debug = debug_file("local");
    let stdout = with_node(
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
    let stdout = distribute(62615, &[], &["--plain", "true", "--param", "value"]);
    assert_contains(
        &stdout,
        "start: Cannot distribute, parameter 'token' cannot be sent to the distant engine, access denied: its value can only be sent over an encrypted connection",
    );

    let debug = debug_file("plain");
    let stdout = with_node(
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
    assert!(!stdout.contains("requester-"), "{}", stdout);
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "references_reach_the_distant_engine_as_locators",
        references_reach_the_distant_engine_as_locators,
    ));
    cases.push((
        "values_are_resolved_by_the_requesting_engine",
        values_are_resolved_by_the_requesting_engine,
    ));
    cases.push(("local_secrets_are_refused", local_secrets_are_refused));
    cases.push((
        "values_are_refused_over_plain_tcp",
        values_are_refused_over_plain_tcp,
    ));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
