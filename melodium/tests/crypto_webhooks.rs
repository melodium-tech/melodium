//! A server verifies webhooks with the `crypto` package: the HMAC signature of GitHub
//! webhooks, and the token of GitLab webhooks, given as a secret by the HTTP server and
//! compared with the expected one without revealing either to the program.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

const SERVER: &str = r#"#!/usr/bin/env melodium
#! name = crypto_webhooks
#! version = 0.1.0
#! require = std:0.11.0 http:0.11.0 net:0.11.0 encoding:0.11.0 crypto:0.11.0

use std/engine/util::startup
use std/engine/log::logInfo
use std/flow::emit
use std/flow::stream
use std/data/map::|get
use std/data/string_map::StringMap
use std/data/string_map::|map
use std/data/string_map/block::get as getHeader
use std/ops/option/block::unwrapOr
use http/server::HttpServer
use http/server::start
use http/server::connection
use http/server::@HttpRequest
use http/method::|post
use http/status::|ok
use http/status::HttpStatus
use net/ip::|localhost_ipv4
use net/ip::|from_ipv4
use encoding::encode
use crypto/hmac::verify
use crypto/token::check

treatment main(
    const port: u16,
    const key: Secret<string> = "env:MELODIUM_SECRET_WEBHOOK_KEY",
    const token: Secret<string> = "env:MELODIUM_SECRET_WEBHOOK_TOKEN"
)
  model server: HttpServer(host=|from_ipv4(|localhost_ipv4()), port=port, secret_headers="X-Gitlab-Token")
{
    startup()
    start[http_server=server]()
    startup.trigger -> start.trigger

    github[http_server=server](key=key)
    gitlab[http_server=server](token=token)
}

treatment respond()
  input  trigger: Block<void>
  output status:  Block<HttpStatus>
  output headers: Block<StringMap>
  output data:    Stream<byte>
{
    emitStatus:  emit<HttpStatus>(value=|ok())
    emitHeaders: emit<StringMap>(value=|map([]))
    reply:   emit<string>(value="received")
    asBytes: stream<string>()
    encoded: encode()
    Self.trigger -> emitStatus.trigger,emit -> Self.status
    Self.trigger -> emitHeaders.trigger,emit -> Self.headers
    Self.trigger -> reply.trigger,emit -> asBytes.block,stream -> encoded.text,data -> Self.data
}

// Verifies the `X-Hub-Signature-256` HMAC signature of the body.
treatment github[http_server: HttpServer](const key: Secret<string>)
{
    connection[http_server=http_server](method=|post(), route="/github")
    respond()
    connection.started -> respond.trigger
    respond.status -> connection.status
    respond.headers -> connection.headers
    respond.data -> connection.data

    signature: getHeader(key="X-Hub-Signature-256")
    verify(key=key, prefix="sha256=")
    connection.headers -> signature.map,value -> verify.signature
    connection.data -> verify.data

    validMessage:   emit<string>(value="valid")
    invalidMessage: emit<string>(value="invalid")
    logValid:   logInfo(label="github")
    logInvalid: logInfo(label="github")
    verify.valid -> validMessage.trigger,emit -> logValid.message
    verify.invalid -> invalidMessage.trigger,emit -> logInvalid.message
}

// Checks the `X-Gitlab-Token` header, given as a secret only.
treatment gitlab[http_server: HttpServer](const token: Secret<string>)
{
    connection[http_server=http_server](method=|post(), route="/gitlab")
    respond()
    connection.started -> respond.trigger
    respond.status -> connection.status
    respond.headers -> connection.headers
    respond.data -> connection.data

    received()
    check(token=token)
    connection.started -> received.trigger,token -> check.received

    validMessage:   emit<string>(value="valid")
    invalidMessage: emit<string>(value="invalid")
    logValid:   logInfo(label="gitlab")
    logInvalid: logInfo(label="gitlab")
    check.valid -> validMessage.trigger,emit -> logValid.message
    check.invalid -> invalidMessage.trigger,emit -> logInvalid.message

    plain: getHeader(key="X-Gitlab-Token")
    plainOrAbsent: unwrapOr<string>(default="absent")
    logPlain: logInfo(label="plain gitlab token")
    connection.headers -> plain.map,value -> plainOrAbsent.option,value -> logPlain.message
}

treatment received()
  require @HttpRequest
  input  trigger: Block<void>
  output token:   Block<Option<Secret<string>>>
{
    emitToken: emit<Option<Secret<string>>>(value=|get<Secret<string>>(@HttpRequest[secret_headers], "X-Gitlab-Token"))
    Self.trigger -> emitToken.trigger,emit -> Self.token
}
"#;

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

#[test]
fn webhooks_are_verified() {
    let port = 62720;
    let server = Command::new(env!("CARGO_BIN_EXE_melodium"))
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
    assert!(Command::new("kill")
        .args(["-INT", &server.id().to_string()])
        .status()
        .unwrap()
        .success());
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
