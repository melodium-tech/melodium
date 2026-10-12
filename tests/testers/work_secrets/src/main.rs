//! The worker request reveals the API token and the container pull secrets only to send
//! them to the API, and fails explicitly when they cannot be resolved.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::time::Duration;

const SCRIPT: &str = include_str!("../../../pull_secret.mel");

const PULL_SECRET: &str = r#"{"auths":{"registry":{"auth":"cHVsbC1zZW50aW5lbA=="}}}"#;

fn run(api_url: &str, envs: &[(&str, &str)]) -> String {
    let script =
        std::env::temp_dir().join(format!("melodium_work_secrets_{}.mel", std::process::id()));
    std::fs::write(&script, SCRIPT).unwrap();
    let output = Command::new("melodium")
        .arg(&script)
        .args(["--api_url", api_url])
        .env_remove("MELODIUM_SECRET_PULL_TEST_SECRET")
        .env_remove("MELODIUM_SECRET_PULL_TEST_API_TOKEN")
        .envs(envs.iter().cloned())
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn credentials_are_revealed_to_reach_the_api() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let api_url = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        while let Ok(n) = stream.read(&mut buffer) {
            if n == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..n]);
            let text = String::from_utf8_lossy(&request);
            if let Some((head, body)) = text.split_once("\r\n\r\n") {
                let length = head
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|length| length.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if body.len() >= length {
                    break;
                }
            }
        }
        let _ =
            stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n");
        String::from_utf8_lossy(&request).into_owned()
    });

    let stdout = run(
        &api_url,
        &[
            ("MELODIUM_SECRET_PULL_TEST_SECRET", PULL_SECRET),
            ("MELODIUM_SECRET_PULL_TEST_API_TOKEN", "t0ken-sentinel"),
        ],
    );
    let request = server.join().unwrap();
    assert!(
        request.contains("authorization: Bearer t0ken-sentinel")
            || request.contains("Authorization: Bearer t0ken-sentinel"),
        "{}",
        request
    );
    assert!(
        request.contains(&PULL_SECRET.replace('"', "\\\"")),
        "{}",
        request
    );
    assert!(!stdout.contains("t0ken-sentinel"), "{}", stdout);
}

fn unresolved_credentials_fail_the_request() {
    let stdout = run("http://127.0.0.1:9", &[]);
    assert!(
        stdout.contains("distant: pull secret: resolution failed"),
        "{}",
        stdout
    );

    let stdout = run(
        "http://127.0.0.1:9",
        &[("MELODIUM_SECRET_PULL_TEST_SECRET", PULL_SECRET)],
    );
    assert!(
        stdout.contains("distant: API token: resolution failed"),
        "{}",
        stdout
    );
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "credentials_are_revealed_to_reach_the_api",
        credentials_are_revealed_to_reach_the_api,
    ));
    cases.push((
        "unresolved_credentials_fail_the_request",
        unresolved_credentials_fail_the_request,
    ));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
