//! A server verifies webhooks with the `crypto` package: the HMAC signature of GitHub
//! webhooks, and the token of GitLab webhooks, given as a secret by the HTTP server and
//! compared with the expected one without revealing either to the program.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

const SERVER: &str = include_str!("../../../crypto_webhooks.mel");

const KEY: &str = "webhook-key-sentinel";
const TOKEN: &str = "webhook-token-sentinel";
const BODY: &str = r#"{"action":"opened"}"#;
/// HMAC-SHA256 of `BODY` with `KEY`.
const SIGNATURE: &str = "1cbdd39ecb1c2ec3658bd0ddece3dd1f582ac759795fce4b9ee0337bcaed3e48";

fn file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_crypto_webhooks_{}_{name}",
        std::process::id()
    ));
    std::fs::write(&path, content).unwrap();
    path
}

/// Posts `body` to `path` with `headers`, giving the response.
fn post(port: u16, path: &str, headers: &[(&str, &str)]) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let mut request = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        BODY.len()
    );
    for (name, value) in headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    request.push_str("\r\n");
    request.push_str(BODY);
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    response
}

fn webhooks_are_verified() {
    let port = 62720;
    let server = tester::interruptible(&mut Command::new("melodium"))
        .arg("run")
        .arg(file("server.mel", SERVER))
        .args(["--port", &port.to_string()])
        .env("MELODIUM_SECRET_WEBHOOK_KEY", KEY)
        .env("MELODIUM_SECRET_WEBHOOK_TOKEN", TOKEN)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));

    let signature = format!("sha256={SIGNATURE}");
    let wrong_signature = format!("sha256={}", "0".repeat(64));
    for (path, headers) in [
        ("/github", vec![("X-Hub-Signature-256", signature.as_str())]),
        (
            "/github",
            vec![("X-Hub-Signature-256", wrong_signature.as_str())],
        ),
        ("/github", vec![("X-Hub-Signature-256", SIGNATURE)]),
        ("/github", vec![]),
        ("/gitlab", vec![("X-Gitlab-Token", TOKEN)]),
        ("/gitlab", vec![("X-Gitlab-Token", "webhook-token-wrong")]),
        ("/gitlab", vec![]),
    ] {
        let response = post(port, path, &headers);
        assert!(response.starts_with("HTTP/1.1 200"), "{}", response);
        std::thread::sleep(Duration::from_millis(200));
    }

    std::thread::sleep(Duration::from_secs(1));
    tester::interrupt(&server).unwrap();
    let server = server.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&server.stdout);

    let count = |line: &str| stdout.lines().filter(|l| l.ends_with(line)).count();
    // A valid signature, then a wrong one, one without its prefix, and none.
    assert_eq!(count("github: valid"), 1, "{}", stdout);
    assert_eq!(count("github: invalid"), 3, "{}", stdout);
    // A valid token, then a wrong one, and none.
    assert_eq!(count("gitlab: valid"), 1, "{}", stdout);
    assert_eq!(count("gitlab: invalid"), 2, "{}", stdout);
    // The token is only given as a secret, out of the plain headers.
    assert_eq!(count("plain gitlab token: absent"), 3, "{}", stdout);
    for value in [KEY, TOKEN] {
        assert!(!stdout.contains(value), "{}", stdout);
    }
}

fn main() {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push(("webhooks_are_verified", webhooks_are_verified));
    tester::cases(&cases)
}
