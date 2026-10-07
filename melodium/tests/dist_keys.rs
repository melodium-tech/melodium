//! `melodium dist` takes its keys from files or the environment, and refuses plain TCP
//! on addresses other than loopback ones unless explicitly allowed.

use std::path::PathBuf;
use std::process::{Command, Output};

fn key_file(name: &str, content: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("melodium_dist_keys_{}_{name}", std::process::id()));
    std::fs::write(&path, content).unwrap();
    path
}

fn dist(args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_melodium"));
    command
        .arg("dist")
        .args(["--port", "0", "--wait", "1"])
        .args(args)
        .env_remove("MELODIUM_DIST_RECV_KEY")
        .env_remove("MELODIUM_DIST_SEND_KEY");
    for (name, value) in env {
        command.env(name, value);
    }
    command.output().unwrap()
}

const RECV_KEY: &str = "4b1f6c8e-7d2a-4f0e-9c3b-1a2b3c4d5e6f";
const SEND_KEY: &str = "9e8d7c6b-5a4f-4e3d-8c2b-1a0f9e8d7c6b";

#[test]
fn keys_come_from_files_or_environment() {
    let recv = key_file("recv", &format!("{RECV_KEY}\n"));
    let send = key_file("send", SEND_KEY);

    let output = dist(
        &[
            "--localhost",
            "--recv-key-file",
            recv.to_str().unwrap(),
            "--send-key-file",
            send.to_str().unwrap(),
        ],
        &[],
    );
    assert!(output.status.success(), "{:?}", output);
    assert!(output.stderr.is_empty(), "{:?}", output);

    let output = dist(
        &["--localhost"],
        &[
            ("MELODIUM_DIST_RECV_KEY", RECV_KEY),
            ("MELODIUM_DIST_SEND_KEY", SEND_KEY),
        ],
    );
    assert!(output.status.success(), "{:?}", output);
    assert!(output.stderr.is_empty(), "{:?}", output);
}

#[test]
fn key_errors_are_explicit() {
    let recv = key_file("errors_recv", RECV_KEY);
    let invalid = key_file("invalid", "invalid-key-sentinel");

    let missing = dist(&["--localhost"], &[]);
    assert_eq!(missing.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(stderr.contains("no key given, use --recv-key-file or MELODIUM_DIST_RECV_KEY"));
    assert!(stderr.contains("no key given, use --send-key-file or MELODIUM_DIST_SEND_KEY"));

    let invalid = dist(
        &[
            "--localhost",
            "--recv-key-file",
            invalid.to_str().unwrap(),
            "--send-key",
            SEND_KEY,
        ],
        &[],
    );
    assert_eq!(invalid.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&invalid.stderr);
    assert!(stderr.contains("is not a valid key"), "{}", stderr);
    assert!(!stderr.contains("invalid-key-sentinel"), "{}", stderr);

    let both = dist(
        &[
            "--localhost",
            "--recv-key",
            RECV_KEY,
            "--recv-key-file",
            recv.to_str().unwrap(),
            "--send-key",
            SEND_KEY,
        ],
        &[],
    );
    assert_eq!(both.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&both.stderr)
        .contains("--recv-key and --recv-key-file cannot be given together"));
}

#[test]
fn keys_as_arguments_are_deprecated() {
    let output = dist(
        &[
            "--localhost",
            "--recv-key",
            RECV_KEY,
            "--send-key",
            SEND_KEY,
        ],
        &[],
    );
    assert!(output.status.success(), "{:?}", output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("prefer --recv-key-file or MELODIUM_DIST_RECV_KEY"));
    assert!(stderr.contains("prefer --send-key-file or MELODIUM_DIST_SEND_KEY"));
}

#[test]
fn plain_tcp_is_refused_on_other_than_loopback_unless_allowed() {
    let keys = [
        ("MELODIUM_DIST_RECV_KEY", RECV_KEY),
        ("MELODIUM_DIST_SEND_KEY", SEND_KEY),
    ];

    let refused = dist(&["--ip", "0.0.0.0", "--disable-tls"], &keys);
    assert_eq!(refused.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&refused.stderr).contains(
        "TLS cannot be disabled on 0.0.0.0, as it is not a loopback address, unless --allow-plain-tcp is given"
    ));

    for args in [
        &["--ip", "0.0.0.0", "--disable-tls", "--allow-plain-tcp"][..],
        &["--localhost", "--disable-tls"][..],
    ] {
        let allowed = dist(args, &keys);
        assert!(allowed.status.success(), "{:?}", allowed);
        assert!(String::from_utf8_lossy(&allowed.stderr).contains("TLS is disabled"));
    }
}
