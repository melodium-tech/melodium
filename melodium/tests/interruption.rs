//! SIGINT, SIGTERM and SIGHUP end a run: running tracks get a grace period, and are
//! stopped by a second signal or at its end, killing the child processes they own.
#![cfg(unix)]

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn script(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_interruption_{}_{name}.mel",
        std::process::id()
    ));
    std::fs::write(&path, content).unwrap();
    path
}

/// Program running `sleep <duration>` as child process, the duration identifying it.
fn sleeper(duration: &str) -> PathBuf {
    script(
        duration,
        &format!(
            r#"#!/usr/bin/env melodium
#! name = sleeper
#! version = 0.11.0
#! require = std:0.11.0 process:0.11.0

use std/engine/util::startup
use process/command::|command
use process/local::execOnce

treatment main()
{{
    startup()
    sleep: execOnce(command=|command("sleep", ["{duration}"]))

    startup.trigger -> sleep.launch
}}
"#
        ),
    )
}

fn sleeping_children(duration: &str) -> usize {
    let output = Command::new("ps").args(["-eo", "args"]).output().unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| *line == format!("sleep {duration}"))
        .count()
}

fn run(script: &PathBuf) -> Child {
    let child = Command::new(env!("CARGO_BIN_EXE_melodium"))
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(3));
    child
}

fn signal(child: &Child, signal: &str) {
    assert!(Command::new("kill")
        .args([signal, &child.id().to_string()])
        .status()
        .unwrap()
        .success());
}

#[test]
fn second_signal_stops_right_away() {
    let duration = "61.1";
    let child = run(&sleeper(duration));
    assert_eq!(sleeping_children(duration), 1);

    let start = Instant::now();
    signal(&child, "-INT");
    std::thread::sleep(Duration::from_millis(500));
    signal(&child, "-INT");
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(130));
    assert!(start.elapsed() < Duration::from_secs(5));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("interrupted by SIGINT, ending within 10 seconds"),
        "{}",
        stdout
    );
    assert!(
        stdout.contains("interrupted again by SIGINT, stopping"),
        "{}",
        stdout
    );
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(sleeping_children(duration), 0);
}

#[test]
fn running_tracks_are_stopped_after_the_grace_period() {
    let duration = "61.2";
    let child = run(&sleeper(duration));

    let start = Instant::now();
    signal(&child, "-TERM");
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(143));
    assert!(start.elapsed() >= Duration::from_secs(10));
    assert!(start.elapsed() < Duration::from_secs(15));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("still running at the end of the grace period, stopping"),
        "{}",
        stdout
    );
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(sleeping_children(duration), 0);
}

#[test]
fn models_end_within_the_grace_period() {
    let server = script(
        "server",
        r#"#!/usr/bin/env melodium
#! name = server
#! version = 0.11.0
#! require = http:0.11.0 net:0.11.0 std:0.11.0

use http/server::HttpServer
use http/server::start
use net/ip::|localhost_ipv4
use net/ip::|from_ipv4
use std/engine/util::startup

treatment main()
  model server: HttpServer(
    host=|from_ipv4(|localhost_ipv4()),
    port=62417
  )
{
    startup()
    start[http_server=server]()

    startup.trigger -> start.trigger
}
"#,
    );
    let child = run(&server);

    let start = Instant::now();
    signal(&child, "-HUP");
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(129));
    assert!(start.elapsed() < Duration::from_secs(5));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("interrupted by SIGHUP"), "{}", stdout);
    assert!(!stdout.contains("stopping"), "{}", stdout);
}
