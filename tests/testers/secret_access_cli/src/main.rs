//! Secret parameters take locators on the command line, and `--check-secrets` resolves
//! them before running.

use std::{path::PathBuf, process::Command};

const SCRIPT: &str = include_str!("../../../../melodium/tests/engine/scripts/secret_access.mel");

fn temp_file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_secret_access_cli_{}_{name}",
        std::process::id()
    ));
    std::fs::write(&path, content).unwrap();
    path
}

fn secret_parameters_take_locators_on_the_command_line() {
    let script = temp_file("script.mel", SCRIPT);
    let password = temp_file("cli_password", "password-sentinel\n");
    let token = temp_file("cli_token", "token-sentinel");
    let missing = std::env::temp_dir().join(format!(
        "melodium_secret_access_cli_{}_missing",
        std::process::id()
    ));

    let output = Command::new("melodium")
        .args(["run", "--check-secrets"])
        .arg(&script)
        .arg("--password")
        .arg(format!("file:{}", password.display()))
        .arg("--token")
        .arg(format!("file:{}", token.display()))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{}", stdout);
    assert!(!stdout.contains("token-sentinel"), "{}", stdout);
    assert!(!stdout.contains("password-sentinel"), "{}", stdout);
    // Names are quoted and escaped, as backslashes of Windows paths are.
    assert!(
        stdout.contains(&format!(
            "secret {:?} denied",
            format!("file:{}", password.display())
        )),
        "{}",
        stdout
    );
    assert!(
        stdout.contains("secrets from 'file:' cannot be plainly revealed"),
        "{}",
        stdout
    );

    let output = Command::new("melodium")
        .args(["run", "--check-secrets"])
        .arg(&script)
        .arg("--password")
        .arg(format!("file:{}", missing.display()))
        .arg("--token")
        .arg(format!("file:{}", token.display()))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Secret of parameter 'password' cannot be resolved"),
        "{}",
        stderr
    );

    let output = Command::new("melodium")
        .arg(&script)
        .args(["--password", "hunter2", "--token", "env:UNUSED"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("is not a secret locator"));
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "secret_parameters_take_locators_on_the_command_line",
        secret_parameters_take_locators_on_the_command_line,
    ));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
