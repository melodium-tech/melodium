//! The HTTP client adds secret headers, revealed by the treatment making each request,
//! and the HTTP server gives the `Authorization` and `Cookie` headers of incoming requests
//! as secrets only, out of the plain request headers.

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

const SERVER: &str = include_str!("../../../http_secrets_server.mel");

const CLIENT: &str = include_str!("../../../http_secrets_client.mel");

// The same request, with the token received at runtime, as `vault::get` gives it, and
// formatted into the header by treatments.
const RUNTIME_CLIENT: &str = include_str!("../../../http_secrets_runtime_client.mel");

const TOKEN: &str = "t0ken-sentinel-value";

fn file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "melodium_http_secrets_{}_{name}",
        std::process::id()
    ));
    std::fs::write(&path, content).unwrap();
    path
}

/// Runs the server, then the client against it, giving their outputs and debug events.
fn exchange(
    port: u16,
    client_script: &str,
    token: Option<&str>,
) -> (String, String, String, String) {
    let server_debug = file(&format!("server_{port}.json"), "");
    let client_debug = file(&format!("client_{port}.json"), "");

    let server = tester::interruptible(&mut Command::new("melodium"))
        .args(["run", "--debug-level", "detailed", "--debug"])
        .arg(&server_debug)
        .arg(file("server.mel", SERVER))
        .args(["--port", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));

    let mut client = Command::new("melodium");
    client
        .args(["run", "--debug-level", "detailed", "--debug"])
        .arg(&client_debug)
        .arg(file(&format!("client_{port}.mel"), client_script))
        .args(["--url", &format!("http://127.0.0.1:{port}/check")])
        .env_remove("MELODIUM_SECRET_HTTP_TEST_TOKEN");
    if let Some(token) = token {
        client.env("MELODIUM_SECRET_HTTP_TEST_TOKEN", token);
    }
    let client: Output = client.output().unwrap();
    assert!(client.status.success(), "{:?}", client);

    std::thread::sleep(Duration::from_secs(1));
    tester::interrupt(&server).unwrap();
    let server = server.wait_with_output().unwrap();

    (
        String::from_utf8_lossy(&server.stdout).into_owned(),
        std::fs::read_to_string(server_debug).unwrap(),
        String::from_utf8_lossy(&client.stdout).into_owned(),
        std::fs::read_to_string(client_debug).unwrap(),
    )
}

fn events(debug: &str) -> Vec<serde_json::Value> {
    serde_json::from_str(debug).unwrap()
}

fn secret_headers_cross_as_secrets() {
    let (server, server_debug, client, client_debug) = exchange(62710, CLIENT, Some(TOKEN));

    // The client reveals the header for the request, and never gives it in plain.
    assert!(!client.contains("error"), "{}", client);
    assert!(!client_debug.contains(TOKEN), "{}", client_debug);
    let revealed: Vec<_> = events(&client_debug)
        .into_iter()
        .filter_map(|event| event["kind"].get("secret_revealed").cloned())
        .collect();
    assert!(!revealed.is_empty());
    for event in &revealed {
        assert_eq!(event["element"]["name"], "request", "{}", event);
        assert_eq!(event["label"], "request", "{}", event);
    }

    // The server gets it as a secret, out of the plain headers, along with the plain ones.
    assert!(
        server.contains("authorization: <secret \"authorization\">"),
        "{}",
        server
    );
    assert!(server.contains("plain x-client: secret-test"), "{}", server);
    assert!(server.contains("plain authorization: absent"), "{}", server);
    assert!(!server.contains(TOKEN), "{}", server);

    // Only the plain reveal of the server gives the value.
    let values: Vec<_> = events(&server_debug)
        .into_iter()
        .filter_map(|event| event["kind"].get("data_sent").cloned())
        .filter(|data| data["output"]["label"] == "revealAuth")
        .map(|data| data["data"]["values"]["values"][0]["string"].clone())
        .collect();
    assert_eq!(values, vec![format!("Bearer {}", TOKEN)]);
}

fn unresolved_secret_headers_fail_the_request() {
    let (server, _, client, _) = exchange(62711, CLIENT, None);
    assert!(
        client.contains("request: secret header 'Authorization': resolution failed"),
        "{}",
        client
    );
    assert!(!server.contains("authorization:"), "{}", server);
}

fn runtime_secret_headers_reach_the_request() {
    let (server, server_debug, client, client_debug) = exchange(62712, RUNTIME_CLIENT, Some(TOKEN));

    assert!(!client.contains("error"), "{}", client);
    assert!(!client_debug.contains(TOKEN), "{}", client_debug);
    assert!(
        server.contains("authorization: <secret \"authorization\">"),
        "{}",
        server
    );
    assert!(server.contains("plain x-client: secret-test"), "{}", server);
    let values: Vec<_> = events(&server_debug)
        .into_iter()
        .filter_map(|event| event["kind"].get("data_sent").cloned())
        .filter(|data| data["output"]["label"] == "revealAuth")
        .map(|data| data["data"]["values"]["values"][0]["string"].clone())
        .collect();
    assert_eq!(values, vec![format!("Bearer {}", TOKEN)]);
}

fn main() {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "secret_headers_cross_as_secrets",
        secret_headers_cross_as_secrets,
    ));
    cases.push((
        "unresolved_secret_headers_fail_the_request",
        unresolved_secret_headers_fail_the_request,
    ));
    cases.push((
        "runtime_secret_headers_reach_the_request",
        runtime_secret_headers_reach_the_request,
    ));
    tester::cases(&cases)
}
