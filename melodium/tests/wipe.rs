//! Plaintext held from secrets is wiped on normal end, on error exits and on handled
//! signals, as reported by `MELODIUM_WIPE_TRACE`, memory contents being out of reach
//! from safe Rust.

use std::path::PathBuf;
use std::process::{Command, Output};

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = wipe
#! version = 0.11.0
#! require = std:0.11.0 process:0.11.0

use std/engine/util::startup
use std/flow::emit
use std/engine/log::logInfo
use std/secret::conceal
use std/secret::reveal
use process/command::|command
use process/local::execOnce

treatment main(const sleep: string = "0", token: Secret<string> = "env:WIPE_TEST_TOKEN")
{
    startup()

    emitValue: emit<string>(value="wipe-sentinel-value")
    concealValue: conceal<string>(name="runtime", plain_reveal=true)
    revealValue: reveal<string>()
    logValue: logInfo(label="revealed")

    startup.trigger -> emitValue.trigger,emit -> concealValue.value,secret -> revealValue.secret,value -> logValue.message

    sleeping: execOnce(command=|command("sleep", [sleep]))
    startup.trigger -> sleeping.launch
}
"#;

fn script() -> PathBuf {
    let path = std::env::temp_dir().join(format!("melodium_wipe_{}.mel", std::process::id()));
    std::fs::write(&path, SCRIPT).unwrap();
    path
}

fn command(args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_melodium"));
    command
        .args(args)
        .env("MELODIUM_WIPE_TRACE", "1")
        .env("WIPE_TEST_TOKEN", "unused");
    command
}

/// Gives the numbers of the wipe trace: wiped before exit, and since start.
fn trace(output: &Output) -> (usize, usize) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let line = stderr
        .lines()
        .find(|line| line.starts_with("wipe: "))
        .unwrap_or_else(|| panic!("no wipe trace in {}", stderr));
    let numbers: Vec<usize> = line
        .split_whitespace()
        .filter_map(|word| word.parse().ok())
        .collect();
    (numbers[0], numbers[1])
}

#[test]
fn plaintext_is_wiped_on_normal_end() {
    let script = script();
    let output = command(&[script.to_str().unwrap()]).output().unwrap();
    assert!(output.status.success(), "{:?}", output);
    // The concealed value and the masking registry.
    let (_, since_start) = trace(&output);
    assert!(since_start >= 2, "{:?}", output);
}

#[test]
fn wiping_runs_on_error_exits() {
    let script = script();
    let output = command(&["run", "--check-secrets", script.to_str().unwrap()])
        .env_remove("WIPE_TEST_TOKEN")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{:?}", output);
    trace(&output);
}

#[cfg(unix)]
#[test]
fn plaintext_is_wiped_on_handled_signals() {
    use std::process::Stdio;
    use std::time::Duration;

    let script = script();
    let child = command(&[script.to_str().unwrap(), "--sleep", "62.7"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(3));
    for _ in 0..2 {
        assert!(Command::new("kill")
            .args(["-INT", &child.id().to_string()])
            .status()
            .unwrap()
            .success());
        std::thread::sleep(Duration::from_millis(500));
    }
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(130), "{:?}", output);
    let (_, since_start) = trace(&output);
    assert!(since_start >= 2, "{:?}", output);
}
