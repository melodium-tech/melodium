//! A revealed value appearing in a log line or in an error message produced by
//! an element is masked in every log output. The value is concealed by the program,
//! as secrets from files and the environment are never plainly revealed.

use std::{path::PathBuf, process::Command};

const SCRIPT: &str = include_str!("../../../secret_masking.mel");

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "melodium_secret_masking_{}_{name}",
        std::process::id()
    ))
}

fn revealed_values_are_masked_in_every_log_output() {
    let script = temp_path("script.mel");
    std::fs::write(&script, SCRIPT).unwrap();
    let logs = temp_path("logs");

    let output = Command::new("melodium")
        .arg("run")
        .arg("--logs")
        .arg(&logs)
        .arg(&script)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let logs = std::fs::read_to_string(&logs).unwrap();

    let error = "request: error sending request for url (<secret \"url\">): connection refused";
    let logged = "url: <secret \"url\">";
    for output in [&*stdout, &*logs] {
        assert!(output.contains(error), "{}", output);
        assert!(output.contains(logged), "{}", output);
    }
    for output in [&*stdout, &*stderr, &*logs] {
        assert!(!output.contains("masking-sentinel"), "{}", output);
    }
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "revealed_values_are_masked_in_every_log_output",
        revealed_values_are_masked_in_every_log_output,
    ));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
