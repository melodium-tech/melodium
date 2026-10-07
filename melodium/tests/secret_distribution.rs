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

use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = secret_distribution
#! version = 0.1.0
#! require = std:0.11.0 net:0.11.0 work:0.11.0 distrib:0.11.0

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
        |entry<Option<Secret<string>>>("token", |locate<string>("env:MELODIUM_SECRET_DIST_PARAM_TOKEN", "param_token", param, true))
    ]))
    accessBlock.emit -> distribStart.access
    logStartError: logError(label="start")
    distribStart.error -> logStartError.message

    dist: distribute[distributor=distributor]()
    distribStart.ready -> dist.trigger

    sendTrigger: sendBlock<void>[distributor=distributor](name="trigger")
    dist.distribution_id -> sendTrigger.distribution_id
    distribStart.ready -> sendTrigger.data

    emitData: emit<Option<Secret<string>>>(value=|locate<string>("env:MELODIUM_SECRET_DIST_DATA_TOKEN", "data_token", data, true))
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

/// Sends a process environment with secrets to a node, that runs a command with it.
const ENVIRONMENT_SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = secret_environment
#! version = 0.1.0
#! require = std:0.11.0 net:0.11.0 work:0.11.0 distrib:0.11.0 process:0.11.0

use std/engine/util::startup
use std/engine/log::logError
use std/flow::emit
use std/data/map::|map as |secret_map
use std/data/map::|entry as |secret_entry
use std/data/string_map::|map
use std/ops/option::|wrap
use std/ops/option::|unwrap_or
use std/ops/option/block::unwrap
use std/secret::|locate
use std/secret::|from_environment
use net/ip::|localhost_ipv4
use net/ip::|from_ipv4
use work/access::Access
use work/access::|new_access
use distrib::DistributionEngine
use distrib::start
use distrib::distribute
use distrib::sendBlock
use distrib::recvBlock
use process/command::Command
use process/command::|command
use process/environment::Environment
use process/environment::|environment
use process/environment::|with_secret_variables
use process/environment::|with_secret_stdin
use process/exec::Executor
use process/exec::execOne
use process/local::|local_executor

model Runner() : DistributionEngine {
    treatment = "secret_environment::run"
    version   = "0.1.0"
}

treatment main(
    const port: u16,
    const send_key: string,
    const recv_key: string,
    const directory: string,
    const transmission: string = "value"
)
  model distributor: Runner()
{
    startup()
    accessBlock: emit<Access>(value=|new_access([|from_ipv4(|localhost_ipv4())], port, send_key, recv_key))
    startup.trigger -> accessBlock.trigger

    distribStart: start[distributor=distributor](params=|secret_map([|secret_entry<string>("directory", directory)]))
    accessBlock.emit -> distribStart.access
    logStartError: logError(label="start")
    distribStart.error -> logStartError.message

    dist: distribute[distributor=distributor]()
    distribStart.ready -> dist.trigger

    sendTrigger: sendBlock<void>[distributor=distributor](name="trigger")
    dist.distribution_id -> sendTrigger.distribution_id
    distribStart.ready -> sendTrigger.data

    emitEnvironment: emit<Option<Environment>>(value=|wrap<Environment>(|with_secret_stdin(
        |with_secret_variables(
            |environment(|map([]), _, false, false),
            |secret_map([|secret_entry<Secret<string>>("TOKEN", |unwrap_or<Secret<string>>(
                |locate<string>("env:MELODIUM_SECRET_DIST_ENV_TOKEN", "token", transmission, false),
                |from_environment("MELODIUM_SECRET_DIST_ENV_TOKEN", "token")
            ))])
        ),
        |unwrap_or<Secret<string>>(
            |locate<string>("env:MELODIUM_SECRET_DIST_ENV_INPUT", "input", transmission, false),
            |from_environment("MELODIUM_SECRET_DIST_ENV_INPUT", "input")
        )
    )))
    sendEnvironment: sendBlock<Option<Environment>>[distributor=distributor](name="environment")
    dist.distribution_id -> sendEnvironment.distribution_id
    distribStart.ready -> emitEnvironment.trigger,emit -> sendEnvironment.data

    recvError: recvBlock<string>[distributor=distributor](name="error")
    dist.distribution_id -> recvError.distribution_id
    logRemoteError: logError(label="remote")
    recvError.data -> logRemoteError.message
}

// Executed by the distant engine: writes the `TOKEN` variable and the standard input to files.
treatment run(const directory: string)
  input  trigger:     Block<void>
  input  environment: Block<Option<Environment>>
  output error:       Block<string>
{
    emitExecutor: emit<Option<Executor>>(value=|local_executor())
    unwrapExecutor: unwrap<Executor>()
    emitCommand: emit<Command>(value=|command("sh", ["-c", "printf '%s' \"$TOKEN\" > \"$0/variable\"; cat > \"$0/input\"", directory]))
    execOne()

    Self.trigger -> emitExecutor.trigger,emit -> unwrapExecutor.option,value -> execOne.executor
    Self.trigger -> emitCommand.trigger,emit -> execOne.command
    Self.environment -> execOne.environment
    execOne.error -> Self.error
}
"#;

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
    let mut command = Command::new(env!("CARGO_BIN_EXE_melodium"));
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
    let output: Output = Command::new(env!("CARGO_BIN_EXE_melodium"))
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

#[test]
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

#[test]
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

#[test]
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

#[test]
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

#[test]
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
