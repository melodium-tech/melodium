//! Secret variables and the secret input of an environment reach the command run, revealed
//! by the treatment running it, and never through command arguments.

use std::path::PathBuf;
use std::process::{Command, Output};

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = process_secrets
#! version = 0.1.0
#! require = std:0.11.0 process:0.11.0

use std/engine/util::startup
use std/engine/log::logError
use std/data/map::Map
use std/data/map::|map as |secret_map
use std/data/map::|entry as |secret_entry
use std/data/string_map::|map
use std/ops::|condition
use std/ops/option::|wrap
use std/secret::|from_environment
use std/secret::|locate
use process/command::|command
use process/environment::Environment
use process/environment::|environment
use process/environment::|with_secret_variables
use process/environment::|with_secret_stdin
use process/local::execOnce

// Writes the `TOKEN` variable and the standard input to files in `directory`.
treatment main(const directory: string, const plain_entry: bool = false)
{
    startup()
    run: execOnce(
        command=|command("sh", ["-c", "printf '%s' \"$TOKEN\" > \"$0/variable\"; cat > \"$0/input\"", directory]),
        environment=|wrap<Environment>(|with_secret_stdin(
            |with_secret_variables(
                |environment(|map([]), _, false, false),
                |condition<Map>(
                    plain_entry,
                    |secret_map([|secret_entry<string>("PLAIN", "not a secret")]),
                    |secret_map([
                        |secret_entry<Option<Secret<string>>>("TOKEN", |locate<string>("env:MELODIUM_SECRET_PROCESS_TEST_TOKEN", "token", "local", false))
                    ])
                )
            ),
            |from_environment("MELODIUM_SECRET_PROCESS_TEST_INPUT", "input")
        ))
    )
    startup.trigger -> run.launch
    logRunError: logError(label="exec")
    run.error -> logRunError.message
}
"#;

const RUNTIME_SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = process_runtime_secrets
#! version = 0.1.0
#! require = std:0.11.0 process:0.11.0

use std/engine/util::startup
use std/engine/log::logError
use std/flow::emit
use std/data/map/block::entry
use std/data/string_map::|map
use std/ops/option/block::wrap
use std/ops/option/block::unwrap
use std/secret::|from_environment
use process/command::Command
use process/command::|command
use process/environment::Environment
use process/environment::|environment
use process/environment/block::withSecretVariables
use process/environment/block::withSecretStdin
use process/exec::Executor
use process/exec::execOne
use process/local::|local_executor

// Same as `process_secrets`, with secrets received at runtime, as `vault::get` gives them.
treatment main(const directory: string)
{
    startup()

    emitToken: emit<Secret<string>>(value=|from_environment("MELODIUM_SECRET_PROCESS_TEST_TOKEN", "token"))
    emitInput: emit<Secret<string>>(value=|from_environment("MELODIUM_SECRET_PROCESS_TEST_INPUT", "input"))
    tokenEntry: entry<Secret<string>>(key="TOKEN")
    emitBase: emit<Environment>(value=|environment(|map([]), _, false, false))
    withSecretVariables()
    withSecretStdin()
    wrapEnvironment: wrap<Environment>()

    startup.trigger -> emitToken.trigger,emit -> tokenEntry.value,map -> withSecretVariables.secret_variables
    startup.trigger -> emitBase.trigger,emit -> withSecretVariables.base,environment -> withSecretStdin.base,environment -> wrapEnvironment.value,option -> execOne.environment
    startup.trigger -> emitInput.trigger,emit -> withSecretStdin.secret

    emitExecutor: emit<Option<Executor>>(value=|local_executor())
    unwrapExecutor: unwrap<Executor>()
    emitCommand: emit<Command>(value=|command("sh", ["-c", "printf '%s' \"$TOKEN\" > \"$0/variable\"; cat > \"$0/input\"", directory]))
    execOne()
    startup.trigger -> emitExecutor.trigger,emit -> unwrapExecutor.option,value -> execOne.executor
    startup.trigger -> emitCommand.trigger,emit -> execOne.command
    logRunError: logError(label="exec")
    execOne.error -> logRunError.message
}
"#;

fn directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_process_secrets_{}_{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn run(directory: &PathBuf, args: &[&str], envs: &[(&str, &str)]) -> Output {
    run_script(directory, SCRIPT, args, envs)
}

fn run_script(directory: &PathBuf, content: &str, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let script = directory.join("process_secrets.mel");
    std::fs::write(&script, content).unwrap();
    let debug = directory.join("debug.json");
    let output = Command::new(env!("CARGO_BIN_EXE_melodium"))
        .args(["run", "--debug-level", "detailed", "--debug"])
        .arg(&debug)
        .arg(&script)
        .arg("--directory")
        .arg(directory)
        .args(args)
        .env_remove("MELODIUM_SECRET_PROCESS_TEST_TOKEN")
        .env_remove("MELODIUM_SECRET_PROCESS_TEST_INPUT")
        .envs(envs.iter().cloned())
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    output
}

#[test]
fn secrets_reach_the_command() {
    let directory = directory("reach");
    let token = "t0ken \"quoted\" $HOME";
    run(
        &directory,
        &[],
        &[
            ("MELODIUM_SECRET_PROCESS_TEST_TOKEN", token),
            ("MELODIUM_SECRET_PROCESS_TEST_INPUT", "input-sentinel"),
        ],
    );
    assert_eq!(
        std::fs::read_to_string(directory.join("variable")).unwrap(),
        token
    );
    assert_eq!(
        std::fs::read_to_string(directory.join("input")).unwrap(),
        "input-sentinel"
    );

    let debug = std::fs::read_to_string(directory.join("debug.json")).unwrap();
    assert!(!debug.contains("t0ken"), "{}", debug);
    assert!(!debug.contains("input-sentinel"), "{}", debug);
    let events: Vec<serde_json::Value> = serde_json::from_str(&debug).unwrap();
    let revealed: Vec<_> = events
        .iter()
        .filter_map(|event| event["kind"].get("secret_revealed"))
        .map(|event| {
            (
                event["secret_name"].as_str().unwrap(),
                event["element"]["name"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        revealed,
        vec![
            ("token", "execOneTerminable"),
            ("input", "execOneTerminable")
        ]
    );
}

#[test]
fn unresolved_secrets_fail_the_command() {
    let directory = directory("unresolved");
    let output = run(
        &directory,
        &[],
        &[("MELODIUM_SECRET_PROCESS_TEST_INPUT", "input")],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("exec: secret variable 'TOKEN': resolution failed"),
        "{}",
        stdout
    );
    assert!(!directory.join("variable").exists());
}

#[test]
fn plain_values_are_refused_as_secret_variables() {
    let directory = directory("plain");
    let output = run(
        &directory,
        &["--plain_entry", "true"],
        &[("MELODIUM_SECRET_PROCESS_TEST_INPUT", "input")],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("exec: secret variable 'PLAIN' is not a Secret<string>"),
        "{}",
        stdout
    );
    assert!(!directory.join("variable").exists());
}

#[test]
fn runtime_secrets_reach_the_command() {
    let directory = directory("runtime");
    let token = "runtime t0ken $HOME";
    run_script(
        &directory,
        RUNTIME_SCRIPT,
        &[],
        &[
            ("MELODIUM_SECRET_PROCESS_TEST_TOKEN", token),
            (
                "MELODIUM_SECRET_PROCESS_TEST_INPUT",
                "runtime-input-sentinel",
            ),
        ],
    );
    assert_eq!(
        std::fs::read_to_string(directory.join("variable")).unwrap(),
        token
    );
    assert_eq!(
        std::fs::read_to_string(directory.join("input")).unwrap(),
        "runtime-input-sentinel"
    );
    let debug = std::fs::read_to_string(directory.join("debug.json")).unwrap();
    assert!(!debug.contains("t0ken"), "{}", debug);
    assert!(!debug.contains("runtime-input-sentinel"), "{}", debug);
}
