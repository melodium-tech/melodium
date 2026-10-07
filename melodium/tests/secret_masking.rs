//! A revealed value appearing in a log line or in an error message produced by
//! an element is masked in every log output. The value is concealed by the program,
//! as secrets from files and the environment are never plainly revealed.

use std::{path::PathBuf, process::Command};

const SCRIPT: &str = r#"#!/usr/bin/env melodium
#! name = secret_masking
#! version = 0.10.4
#! require = std:0.10.4

use std/engine/util::startup
use std/flow::emit
use std/flow::stream
use std/engine/log::logInfo
use std/engine/log::logErrors
use std/data/string_map::entry
use std/text/compose::format
use std/secret::conceal
use std/secret::reveal

treatment main()
{
    startup()

    emitUrl: emit<string>(value="https://ci:masking-sentinel-token@gitlab.com/group/project.git")
    concealUrl: conceal<string>(name="url", plain_reveal=true)
    revealUrl: reveal<string>()
    logUrl: logInfo(label="url")

    startup.trigger -> emitUrl.trigger,emit -> concealUrl.value,secret -> revealUrl.secret,value -> logUrl.message

    // An element error quoting the value, as an HTTP library would quote the URL.
    stream<string>()
    entry(key="url")
    format(format="error sending request for url ({url}): connection refused")
    logRequestErrors: logErrors(label="request")

    revealUrl.value -> stream.block,stream -> entry.value,map -> format.entries,formatted -> logRequestErrors.messages
}
"#;

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "melodium_secret_masking_{}_{name}",
        std::process::id()
    ))
}

#[test]
fn revealed_values_are_masked_in_every_log_output() {
    let script = temp_path("script.mel");
    std::fs::write(&script, SCRIPT).unwrap();
    let logs = temp_path("logs");

    let output = Command::new(env!("CARGO_BIN_EXE_melodium"))
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
