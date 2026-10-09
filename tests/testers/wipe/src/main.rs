//! Plaintext held from secrets is wiped on normal end, on error exits and on handled
//! signals, as reported by `MELODIUM_WIPE_TRACE`, memory contents being out of reach
//! from safe Rust.

use std::path::PathBuf;
use std::process::{Command, Output};

const SCRIPT: &str = include_str!("../../../wipe.mel");

fn script() -> PathBuf {
    let path = std::env::temp_dir().join(format!("melodium_wipe_{}.mel", std::process::id()));
    std::fs::write(&path, SCRIPT).unwrap();
    path
}

fn command(args: &[&str]) -> Command {
    let mut command = Command::new("melodium");
    command
        .args(args)
        .env("MELODIUM_WIPE_TRACE", "1")
        .env("MELODIUM_SECRET_WIPE_TEST_TOKEN", "unused");
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

fn plaintext_is_wiped_on_normal_end() {
    let script = script();
    let output = command(&[script.to_str().unwrap()])
        .args(tester::powershell("Start-Sleep 0"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    // The concealed value and the masking registry.
    let (_, since_start) = trace(&output);
    assert!(since_start >= 2, "{:?}", output);
}

fn wiping_runs_on_error_exits() {
    let script = script();
    let output = command(&["run", "--check-secrets", script.to_str().unwrap()])
        .env_remove("MELODIUM_SECRET_WIPE_TEST_TOKEN")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{:?}", output);
    trace(&output);
}

#[cfg(unix)]
fn plaintext_is_wiped_on_handled_signals() {
    use std::process::Stdio;
    use std::time::Duration;

    let script = script();
    let child = command(&[script.to_str().unwrap(), "--commands", "sleep 62.7"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(3));
    for _ in 0..2 {
        tester::signal(child.id(), tester::SIGINT).unwrap();
        std::thread::sleep(Duration::from_millis(500));
    }
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(130), "{:?}", output);
    let (_, since_start) = trace(&output);
    assert!(since_start >= 2, "{:?}", output);
}

fn main() {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "plaintext_is_wiped_on_normal_end",
        plaintext_is_wiped_on_normal_end,
    ));
    cases.push(("wiping_runs_on_error_exits", wiping_runs_on_error_exits));
    #[cfg(unix)]
    cases.push((
        "plaintext_is_wiped_on_handled_signals",
        plaintext_is_wiped_on_handled_signals,
    ));
    tester::cases(&cases)
}
