//! The `11_signed_webhooks` tutorial answers webhooks according to their verification:
//! `200 accepted` and the event logged for a valid GitHub signature or GitLab token, `403
//! rejected` otherwise, including when the expected secret cannot be revealed. The
//! secrets never appear in the output.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::time::Duration;

const COMPO: &str = include_str!("../../../../examples/tutorial/11_signed_webhooks/Compo.toml");
const MAIN: &str = include_str!("../../../../examples/tutorial/11_signed_webhooks/main.mel");

const GITHUB_SECRET: &str = "github-secret-sentinel";
const GITLAB_TOKEN: &str = "gitlab-token-sentinel";
const BODY: &str = r#"{"ref":"refs/heads/main"}"#;
/// HMAC-SHA256 of `BODY` with `GITHUB_SECRET`.
const SIGNATURE: &str = "000a0af046024c05be5c1c2bb0be0e95fa39c61f82ad74280b9331fe96751552";

/// Writes the tutorial project to a temporary directory, giving its `Compo.toml`.
fn project(case: &str) -> PathBuf {
    let dir = tester::temp_path("signed_webhooks", case);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.mel"), MAIN).unwrap();
    let compo = dir.join("Compo.toml");
    std::fs::write(&compo, COMPO).unwrap();
    compo
}

/// A running server, killed if a case fails before stopping it, so that it does not
/// answer the requests of a later run on the same port.
struct Server(Option<Child>);

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn serve(case: &str, port: u16, gitlab_token: Option<&str>) -> Server {
    let mut command = tester::melodium();
    tester::interruptible(&mut command)
        .arg("run")
        .arg(project(case))
        .args(["--port", &port.to_string()])
        .env("MELODIUM_SECRET_GITHUB_WEBHOOK_SECRET", GITHUB_SECRET)
        .env_remove("MELODIUM_SECRET_GITLAB_WEBHOOK_TOKEN")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(token) = gitlab_token {
        command.env("MELODIUM_SECRET_GITLAB_WEBHOOK_TOKEN", token);
    }
    let server = Server(Some(command.spawn().unwrap()));
    std::thread::sleep(Duration::from_secs(2));
    server
}

/// Posts `BODY` to `path` with `headers`, giving the status line and the body of the response.
fn post(port: u16, path: &str, headers: &[(&str, &str)]) -> (String, String) {
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
    std::thread::sleep(Duration::from_millis(200));

    let status = response.lines().next().unwrap_or_default().to_string();
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default();
    (status, body)
}

/// Stops `server`, giving everything it printed.
fn stop(mut server: Server) -> String {
    let server = server.0.take().unwrap();
    std::thread::sleep(Duration::from_secs(1));
    tester::interrupt(&server).unwrap();
    let server = server.wait_with_output().unwrap();
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&server.stdout),
        String::from_utf8_lossy(&server.stderr)
    );
    for value in [GITHUB_SECRET, GITLAB_TOKEN] {
        assert!(!output.contains(value), "{}", output);
    }
    output
}

fn count(output: &str, line: &str) -> usize {
    output.lines().filter(|l| l.ends_with(line)).count()
}

fn assert_answer(answer: (String, String), code: &str, text: &str) {
    assert!(
        answer.0.starts_with(&format!("HTTP/1.1 {code}")),
        "{:?}",
        answer
    );
    assert!(answer.1.contains(text), "{:?}", answer);
}

fn webhooks_are_answered_by_their_verification() {
    let port = 62730;
    let server = serve("verification", port, Some(GITLAB_TOKEN));

    let signature = format!("sha256={SIGNATURE}");
    let wrong_signature = format!("sha256={}", "0".repeat(64));
    let github = |signature: Option<&str>| {
        let mut headers = vec![("X-GitHub-Event", "push")];
        if let Some(signature) = signature {
            headers.push(("X-Hub-Signature-256", signature));
        }
        post(port, "/github", &headers)
    };
    assert_answer(github(Some(&signature)), "200", "accepted");
    assert_answer(github(Some(&wrong_signature)), "403", "rejected");
    // Without its `sha256=` prefix.
    assert_answer(github(Some(SIGNATURE)), "403", "rejected");
    assert_answer(github(None), "403", "rejected");

    let gitlab = |token: Option<&str>| {
        let mut headers = vec![("X-Gitlab-Event", "Push Hook")];
        if let Some(token) = token {
            headers.push(("X-Gitlab-Token", token));
        }
        post(port, "/gitlab", &headers)
    };
    assert_answer(gitlab(Some(GITLAB_TOKEN)), "200", "accepted");
    assert_answer(gitlab(Some("gitlab-token-wrong")), "403", "rejected");
    assert_answer(gitlab(None), "403", "rejected");

    let output = stop(server);
    // Events are only logged for accepted requests.
    assert_eq!(count(&output, "github: push"), 1, "{}", output);
    assert_eq!(
        count(&output, "github: invalid signature, request rejected"),
        3,
        "{}",
        output
    );
    assert_eq!(count(&output, "gitlab: Push Hook"), 1, "{}", output);
    assert_eq!(
        count(&output, "gitlab: invalid token, request rejected"),
        2,
        "{}",
        output
    );
}

fn unrevealable_secret_rejects_requests() {
    let port = 62731;
    let server = serve("unrevealable", port, None);

    assert_answer(
        post(
            port,
            "/gitlab",
            &[
                ("X-Gitlab-Event", "Push Hook"),
                ("X-Gitlab-Token", GITLAB_TOKEN),
            ],
        ),
        "403",
        "rejected",
    );
    // The other route is not affected.
    let signature = format!("sha256={SIGNATURE}");
    assert_answer(
        post(
            port,
            "/github",
            &[
                ("X-GitHub-Event", "push"),
                ("X-Hub-Signature-256", &signature),
            ],
        ),
        "200",
        "accepted",
    );

    let output = stop(server);
    assert_eq!(count(&output, "gitlab: Push Hook"), 0, "{}", output);
    assert!(
        output
            .lines()
            .any(|l| l.contains("error: gitlab:")
                && l.contains("MELODIUM_SECRET_GITLAB_WEBHOOK_TOKEN")),
        "{}",
        output
    );
}

fn main() {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "webhooks_are_answered_by_their_verification",
        webhooks_are_answered_by_their_verification,
    ));
    cases.push((
        "unrevealable_secret_rejects_requests",
        unrevealable_secret_rejects_requests,
    ));
    tester::cases(&cases)
}
