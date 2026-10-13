//! `melodium check` rejects programs calling functions with more parameters or generics than they take.

use std::{path::PathBuf, process::Command};

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "melodium_check_rejections_{}_{name}.mel",
        std::process::id()
    ))
}

fn program(name: &str, call: &str) -> String {
    format!(
        r#"#!/usr/bin/env melodium
#! name = {name}
#! version = 0.1.0
#! require = std:0.11.0

use std/engine/util::startup
use std/flow::emit
use std/ops/option::|wrap

treatment main()
{{
    startup()
    emit<Option<u8>>(value={call})
    startup.trigger -> emit.trigger
}}
"#
    )
}

fn check(name: &str, call: &str) -> (bool, String) {
    let path = temp_path(name);
    std::fs::write(&path, program(name, call)).unwrap();
    let output = Command::new("melodium")
        .arg("check")
        .arg(&path)
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&path);
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

fn exact_call_passes() {
    let (success, output) = check("exact_call", "|wrap<u8>(1)");
    assert!(success, "valid call rejected: {output}");
}

fn extra_parameter_rejected() {
    let (success, output) = check("extra_parameter", "|wrap<u8>(1, 2)");
    assert!(!success, "extra parameter accepted");
    assert!(
        output.contains("takes 1 parameters but 2 are given"),
        "unexpected output: {output}"
    );
}

fn extra_generic_rejected() {
    let (success, output) = check("extra_generic", "|wrap<u8, u8>(1)");
    assert!(!success, "extra generic accepted");
    assert!(
        output.contains("takes 1 generics but 2 are given"),
        "unexpected output: {output}"
    );
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push(("exact_call_passes", exact_call_passes));
    cases.push(("extra_parameter_rejected", extra_parameter_rejected));
    cases.push(("extra_generic_rejected", extra_generic_rejected));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
