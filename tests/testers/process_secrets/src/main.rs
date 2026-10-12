//! Secret variables and the secret input of an environment reach the command run, revealed
//! by the treatment running it, and never through command arguments.

use std::path::PathBuf;
use std::process::{Command, Output};

const SCRIPT: &str = include_str!("../../../process_secrets.mel");

const RUNTIME_SCRIPT: &str = include_str!("../../../process_runtime_secrets.mel");

/// The commands of the programs, for Windows.
const POWERSHELL: &str = "[IO.File]::WriteAllText((Join-Path $PWD 'variable'), $env:TOKEN); [IO.File]::WriteAllText((Join-Path $PWD 'input'), [Console]::In.ReadToEnd())";

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
    let output = Command::new("melodium")
        .args(["run", "--debug-level", "detailed", "--debug"])
        .arg(&debug)
        .arg(&script)
        .arg("--directory")
        .arg(directory)
        .args(tester::powershell(POWERSHELL))
        .args(args)
        .env_remove("MELODIUM_SECRET_PROCESS_TEST_TOKEN")
        .env_remove("MELODIUM_SECRET_PROCESS_TEST_INPUT")
        .envs(envs.iter().cloned())
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    output
}

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

fn main() {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push(("secrets_reach_the_command", secrets_reach_the_command));
    cases.push((
        "unresolved_secrets_fail_the_command",
        unresolved_secrets_fail_the_command,
    ));
    cases.push((
        "plain_values_are_refused_as_secret_variables",
        plain_values_are_refused_as_secret_variables,
    ));
    cases.push((
        "runtime_secrets_reach_the_command",
        runtime_secrets_reach_the_command,
    ));
    tester::cases(&cases)
}
