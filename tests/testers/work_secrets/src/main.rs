//! The worker request reveals the API token and the container pull secrets only to send
//! them to the API, and fails explicitly when they cannot be resolved. Secrets of
//! containers stored on Cadence.CI go by name, never revealed, others by value when their
//! policy allows it, and local ones are refused.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::time::Duration;

const SCRIPT: &str = include_str!("../../../pull_secret.mel");
const CONTAINER_SCRIPT: &str = include_str!("../../../container_secrets.mel");

const PULL_SECRET: &str = r#"{"auths":{"registry":{"auth":"cHVsbC1zZW50aW5lbA=="}}}"#;

fn run(api_url: &str, envs: &[(&str, &str)]) -> String {
    run_script("pull_secret", SCRIPT, api_url, &[], envs)
}

fn run_script(
    name: &str,
    content: &str,
    api_url: &str,
    args: &[&str],
    envs: &[(&str, &str)],
) -> String {
    let script = std::env::temp_dir().join(format!(
        "melodium_work_secrets_{}_{name}.mel",
        std::process::id()
    ));
    std::fs::write(&script, content).unwrap();
    let output = Command::new("melodium")
        .arg(&script)
        .args(["--api_url", api_url])
        .args(args)
        .env_remove("MELODIUM_SECRET_PULL_TEST_SECRET")
        .env_remove("MELODIUM_SECRET_PULL_TEST_API_TOKEN")
        .env_remove("MELODIUM_API_URL")
        .env_remove("MELODIUM_API_TOKEN")
        .env_remove("MELODIUM_RUN_ID")
        .envs(envs.iter().cloned())
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Starts a mock of the API answering the first request with an error, giving its address
/// and the request it received.
fn api_server() -> (String, std::thread::JoinHandle<String>) {
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
    (api_url, server)
}

/// Gives the JSON body of a request received by `api_server`.
fn body(request: &str) -> serde_json::Value {
    serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap()
}

fn credentials_are_revealed_to_reach_the_api() {
    let (api_url, server) = api_server();
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
        stdout.contains("distant: pull secret of container 'main': resolution failed"),
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

fn container_secrets_stored_on_cadence_ci_go_by_name() {
    let (api_url, server) = api_server();
    let stdout = run_script(
        "container",
        CONTAINER_SCRIPT,
        &api_url,
        &[],
        &[("MELODIUM_SECRET_PULL_TEST_API_TOKEN", "t0ken-sentinel")],
    );
    let request = body(&server.join().unwrap());
    let service = &request["service_containers"][0];
    assert_eq!(service["pull_secret"], "{}", "{}", stdout);
    assert_eq!(service["pull_secret_name"], "registry/docker");
    assert_eq!(
        service["secret_env"],
        serde_json::json!({"POSTGRES_PASSWORD": "ci/postgres-password"})
    );
    assert_eq!(service["env"], serde_json::json!({"POSTGRES_USER": "ci"}));
}

fn other_container_secrets_go_by_value() {
    let (api_url, server) = api_server();
    let stdout = run_script(
        "container",
        CONTAINER_SCRIPT,
        &api_url,
        &[
            "--password",
            "env:MELODIUM_SECRET_CONTAINER_TEST_PASSWORD",
            "--transmission",
            "value",
        ],
        &[
            ("MELODIUM_SECRET_PULL_TEST_API_TOKEN", "t0ken-sentinel"),
            (
                "MELODIUM_SECRET_CONTAINER_TEST_PASSWORD",
                "p4ssword-sentinel",
            ),
        ],
    );
    let request = body(&server.join().unwrap());
    let service = &request["service_containers"][0];
    assert_eq!(
        service["env"],
        serde_json::json!({"POSTGRES_USER": "ci", "POSTGRES_PASSWORD": "p4ssword-sentinel"}),
        "{}",
        stdout
    );
    assert!(service.get("secret_env").is_none());
    assert!(!stdout.contains("p4ssword-sentinel"), "{}", stdout);
}

fn local_container_secrets_are_refused() {
    for (transmission, reason) in [
        ("local", "access denied: its policy keeps it on this engine"),
        (
            "reference",
            "access denied: containers only get secrets by reference when stored on Cadence.CI, located 'api:<name>'",
        ),
    ] {
        let stdout = run_script(
            "container",
            CONTAINER_SCRIPT,
            "http://127.0.0.1:9",
            &[
                "--password",
                "env:MELODIUM_SECRET_CONTAINER_TEST_PASSWORD",
                "--transmission",
                transmission,
            ],
            &[
                ("MELODIUM_SECRET_PULL_TEST_API_TOKEN", "t0ken-sentinel"),
                ("MELODIUM_SECRET_CONTAINER_TEST_PASSWORD", "p4ssword-sentinel"),
            ],
        );
        assert!(
            stdout.contains(&format!(
                "distant: variable 'POSTGRES_PASSWORD' of service container 'postgres': {reason}"
            )),
            "{}",
            stdout
        );
        assert!(!stdout.contains("p4ssword-sentinel"), "{}", stdout);
    }
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
    cases.push((
        "container_secrets_stored_on_cadence_ci_go_by_name",
        container_secrets_stored_on_cadence_ci_go_by_name,
    ));
    cases.push((
        "other_container_secrets_go_by_value",
        other_container_secrets_go_by_value,
    ));
    cases.push((
        "local_container_secrets_are_refused",
        local_container_secrets_are_refused,
    ));
    tester::cases(&cases)
}

fn main() {
    run_cases();
}
