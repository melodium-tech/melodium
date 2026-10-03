//! Secrets cross to a distant engine following their transmission policy: by reference
//! the distant engine resolves the locator with its own sources, by value the resolved
//! value is sent over TLS only, and local secrets are refused with an explicit error.
//!
//! Each test runs an orchestrating `melodium run` and, when a connection is needed,
//! a `melodium dist --localhost` node, both with different values for the same
//! variables. The node reveals what it receives and sends it back, and values known
//! to the orchestrator are masked in its logs.

use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = secret_distribution
#! version = 0.1.0
#! require = std:0.10.4 net:0.10.4 work:0.10.4 distrib:0.10.4

use std/engine/util::startup
use std/engine/log::logInfo
use std/engine/log::logError
use std/flow::emit
use std/data/map::|map
use std/data/map::|entry
use std/ops::|condition
use std/ops/option/block::unwrap
use std/secret::|locate
use std/secret::reveal
use net/ip::|localhost_ipv4
use net/ip::|from_ipv4
use work/access::Access
use work/access::|new_access
use work/access::|new_plain_access
use distrib::DistributionEngine
use distrib::start
use distrib::stop
use distrib::distribute
use distrib::sendBlock
use distrib::recvBlock

model Prober() : DistributionEngine {
    treatment = "secret_distribution::probe"
    version   = "0.1.0"
}

treatment main(
    const port: u16,
    const send_key: string,
    const recv_key: string,
    const plain: bool = false,
    const param: string = "reference",
    const data: string = "reference"
)
  model distributor: Prober()
{
    startup()

    accessBlock: emit<Access>(value=|condition<Access>(
        plain,
        |new_plain_access([|from_ipv4(|localhost_ipv4())], port, send_key, recv_key),
        |new_access([|from_ipv4(|localhost_ipv4())], port, send_key, recv_key)
    ))
    startup.trigger -> accessBlock.trigger

    distribStart: start[distributor=distributor](params=|map([
        |entry<Option<Secret<string>>>("token", |locate<string>("env:DIST_PARAM_TOKEN", "param_token", param, true))
    ]))
    accessBlock.emit -> distribStart.access
    logStartError: logError(label="start")
    distribStart.error -> logStartError.message

    dist: distribute[distributor=distributor]()
    distribStart.ready -> dist.trigger

    sendTrigger: sendBlock<void>[distributor=distributor](name="trigger")
    dist.distribution_id -> sendTrigger.distribution_id
    distribStart.ready -> sendTrigger.data

    emitData: emit<Option<Secret<string>>>(value=|locate<string>("env:DIST_DATA_TOKEN", "data_token", data, true))
    unwrapData: unwrap<Secret<string>>()
    sendData: sendBlock<Secret<string>>[distributor=distributor](name="data")
    dist.distribution_id -> sendData.distribution_id
    distribStart.ready -> emitData.trigger,emit -> unwrapData.option,value -> sendData.data

    recvParam: recvBlock<string>[distributor=distributor](name="param")
    recvData: recvBlock<string>[distributor=distributor](name="revealed")
    dist.distribution_id -> recvParam.distribution_id
    dist.distribution_id -> recvData.distribution_id
    logParam: logInfo(label="param")
    logData: logInfo(label="revealed")
    recvParam.data -> logParam.message
    recvData.data -> logData.message
}

// Executed by the distant engine: reveals the secrets it was given.
treatment probe(token: Option<Secret<string>>)
  input  trigger:  Block<void>
  input  data:     Block<Secret<string>>
  output param:    Block<string>
  output revealed: Block<string>
{
    emitParam: emit<Option<Secret<string>>>(value=token)
    unwrapParam: unwrap<Secret<string>>()
    revealParam: reveal<string>()
    Self.trigger -> emitParam.trigger,emit -> unwrapParam.option,value -> revealParam.secret,value -> Self.param

    revealData: reveal<string>()
    Self.data -> revealData.secret,value -> Self.revealed
}
"#;

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

/// Starts a node, that ends a few seconds after the orchestrator connects
/// and sends its debug events to it.
fn node(port: u16, tls: bool) -> Child {
    let mut command = Command::new(env!("CARGO_BIN_EXE_melodium"));
    command
        .args(["dist", "--localhost", "--port", &port.to_string()])
        .args(["--debug-level", "basic"])
        .args(["--wait", "30", "--duration", "5"])
        .env("MELODIUM_GROUP_ID", GROUP_ID)
        .env("MELODIUM_DIST_RECV_KEY", NODE_RECV_KEY)
        .env("MELODIUM_DIST_SEND_KEY", NODE_SEND_KEY)
        .env("DIST_PARAM_TOKEN", "node-param-value")
        .env("DIST_DATA_TOKEN", "node-data-value")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if !tls {
        command.arg("--disable-tls");
    }
    let child = command.spawn().unwrap();
    std::thread::sleep(Duration::from_secs(3));
    child
}

fn orchestrate(port: u16, options: &[&str], args: &[&str]) -> String {
    let output: Output = Command::new(env!("CARGO_BIN_EXE_melodium"))
        .arg("run")
        .args(options)
        .arg(script())
        .args(["--port", &port.to_string()])
        .args(["--send_key", NODE_RECV_KEY, "--recv_key", NODE_SEND_KEY])
        .args(args)
        .env("MELODIUM_GROUP_ID", GROUP_ID)
        .env("DIST_PARAM_TOKEN", "orchestrator-param-value")
        .env("DIST_DATA_TOKEN", "orchestrator-data-value")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn with_node(port: u16, tls: bool, options: &[&str], args: &[&str]) -> String {
    let mut node = node(port, tls);
    let stdout = orchestrate(port, options, args);
    let _ = node.kill();
    let _ = node.wait();
    stdout
}

fn assert_contains(stdout: &str, expected: &str) {
    assert!(
        stdout.contains(expected),
        "expected {:?} in:\n{}",
        expected,
        stdout
    );
}

#[test]
fn references_are_resolved_by_the_distant_engine() {
    let stdout = with_node(62611, true, &[], &[]);
    assert_contains(&stdout, "param: node-param-value");
    assert_contains(&stdout, "revealed: node-data-value");
    assert!(!stdout.contains("error"), "{}", stdout);
}

#[test]
fn values_are_resolved_by_the_orchestrating_engine() {
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

#[test]
fn local_secrets_are_refused() {
    // Parameters are refused before connecting.
    let stdout = orchestrate(62613, &[], &["--param", "local"]);
    assert_contains(
        &stdout,
        "start: Cannot distribute, parameter 'token' cannot be sent to the distant engine, access denied: its policy keeps it on this engine",
    );

    let stdout = with_node(62614, true, &[], &["--data", "local"]);
    assert_contains(
        &stdout,
        "distrib: Cannot send 'data' to the distant engine, access denied: its policy keeps it on this engine",
    );
    assert_contains(&stdout, "param: node-param-value");
    assert!(!stdout.contains("revealed:"), "{}", stdout);
}

#[test]
fn values_are_refused_over_plain_tcp() {
    let stdout = orchestrate(62615, &[], &["--plain", "true", "--param", "value"]);
    assert_contains(
        &stdout,
        "start: Cannot distribute, parameter 'token' cannot be sent to the distant engine, access denied: its value can only be sent over an encrypted connection",
    );

    let stdout = with_node(62616, false, &[], &["--plain", "true", "--data", "value"]);
    assert_contains(
        &stdout,
        "distrib: Cannot send 'data' to the distant engine, access denied: its value can only be sent over an encrypted connection",
    );
    assert_contains(&stdout, "param: node-param-value");
    assert!(!stdout.contains("orchestrator-"), "{}", stdout);
}
