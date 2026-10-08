//! The HTTP client adds secret headers, revealed by the treatment making each request,
//! and the HTTP server gives the `Authorization` and `Cookie` headers of incoming requests
//! as secrets only, out of the plain request headers.
#![cfg(unix)]

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

const SERVER: &str = r#"#!/usr/bin/env melodium
#! name = http_secrets_server
#! version = 0.1.0
#! require = std:0.11.0 http:0.11.0 net:0.11.0 encoding:0.11.0

use std/engine/util::startup
use std/engine/log::logInfo
use std/engine/log::logInfos
use std/data/string_map::get
use std/ops/option::unwrapOr
use std/flow::emit
use std/flow::stream
use std/data/string_map::StringMap
use std/data/string_map::|map
use std/ops/option/block::unwrap
use std/secret::reveal
use http/server::HttpServer
use http/server::start
use http/server::connection
use http/server::@HttpRequest
use http/method::|get
use http/status::|ok
use http/status::HttpStatus
use net/ip::|localhost_ipv4
use net/ip::|from_ipv4
use encoding::encode

treatment main(const port: u16)
  model server: HttpServer(host=|from_ipv4(|localhost_ipv4()), port=port)
{
    startup()
    start[http_server=server]()
    startup.trigger -> start.trigger

    check[http_server=server]()
}

treatment check[http_server: HttpServer]() {
    connection[http_server=http_server](method=|get(), route="/check")

    status:  emit<HttpStatus>(value=|ok())
    headers: emit<StringMap>(value=|map([]))
    connection.started -> status.trigger,emit  -> connection.status
    connection.started -> headers.trigger,emit -> connection.headers

    reply:   emit<string>(value="checked")
    asBytes: stream<string>()
    encoded: encode()
    connection.started -> reply.trigger,emit -> asBytes.block,stream -> encoded.text,data -> connection.data

    authorization()
    connection.started -> authorization.trigger

    // Headers given in plain.
    headersStream: stream<StringMap>()
    connection.headers -> headersStream.block
    plainClient: get(key="X-Client")
    plainAuthorization: get(key="Authorization")
    clientOrAbsent: unwrapOr<string>(default="absent")
    authorizationOrAbsent: unwrapOr<string>(default="absent")
    logClient: logInfos(label="plain x-client")
    logAuthorization: logInfos(label="plain authorization")
    headersStream.stream -> plainClient.map,value -> clientOrAbsent.option,value -> logClient.messages
    headersStream.stream -> plainAuthorization.map,value -> authorizationOrAbsent.option,value -> logAuthorization.messages
}

// Reveals the `Authorization` header of the request, given as a secret.
treatment authorization()
  require @HttpRequest
  input trigger: Block<void>
{
    emitAuth: emit<Option<Secret<string>>>(value=@HttpRequest[authorization])
    unwrapAuth: unwrap<Secret<string>>()
    revealAuth: reveal<string>()
    logAuth: logInfo(label="authorization")
    Self.trigger -> emitAuth.trigger,emit -> unwrapAuth.option,value -> revealAuth.secret,value -> logAuth.message
}
"#;

const CLIENT: &str = r#"#!/usr/bin/env melodium
#! name = http_secrets_client
#! version = 0.1.0
#! require = std:0.11.0 http:0.11.0

use std/engine/util::startup
use std/engine/log::logError
use std/flow::emit
use std/data/map::Map
use std/data/map::|map as |secret_map
use std/data/map::|entry as |secret_entry
use std/data/string_map::StringMap
use std/data/string_map::|map
use std/data/string_map::|entry
use std/ops/option::|wrap
use std/secret::|format as |secret_format
use std/secret::|from_environment
use http/client::HttpClient
use http/client::get

treatment main(const url: string)
  model client: HttpClient(
      base_url=_,
      headers=|map([|entry("X-Client", "secret-test")]),
      secret_headers=|wrap<Map>(|secret_map([
          |secret_entry<Secret<string>>("Authorization", |secret_format(
              "Bearer {token}",
              |secret_map([|secret_entry<Secret<string>>("token", |from_environment("MELODIUM_SECRET_HTTP_TEST_TOKEN", "token"))]),
              "authorization"
          ))
      ]))
  )
{
    startup()
    emitUrl: emit<string>(value=url)
    noHeaders: emit<StringMap>(value=|map([]))
    request: get[client=client]()
    startup.trigger -> emitUrl.trigger,emit -> request.url
    startup.trigger -> noHeaders.trigger,emit -> request.headers
    logRequestError: logError(label="request")
    request.error -> logRequestError.message
}
"#;

// The same request, with the token received at runtime, as `vault::get` gives it, and
// formatted into the header by treatments.
const RUNTIME_CLIENT: &str = r#"#!/usr/bin/env melodium
#! name = http_secrets_runtime_client
#! version = 0.1.0
#! require = std:0.11.0 http:0.11.0

use std/engine/util::startup
use std/engine/log::logError
use std/flow::emit
use std/data/map/block::entry
use std/data/string_map::StringMap
use std/data/string_map::|map
use std/data/string_map::|entry
use std/secret::|from_environment
use std/secret/block::format
use http/client::HttpClient
use http/client::requestWithSecretHeaders
use http/method::|get

treatment main(const url: string)
  model client: HttpClient(base_url=_, headers=|map([|entry("X-Client", "secret-test")]))
{
    startup()
    emitToken: emit<Secret<string>>(value=|from_environment("MELODIUM_SECRET_HTTP_TEST_TOKEN", "token"))
    tokenEntry: entry<Secret<string>>(key="token")
    bearer: format(template="Bearer {token}", name="authorization")
    headerEntry: entry<Secret<string>>(key="Authorization")
    startup.trigger -> emitToken.trigger,emit -> tokenEntry.value,map -> bearer.entries,secret -> headerEntry.value

    emitUrl: emit<string>(value=url)
    noHeaders: emit<StringMap>(value=|map([]))
    request: requestWithSecretHeaders[client=client](method=|get())
    startup.trigger -> emitUrl.trigger,emit -> request.url
    startup.trigger -> noHeaders.trigger,emit -> request.req_headers
    headerEntry.map -> request.secret_headers
    logRequestError: logError(label="request")
    request.error -> logRequestError.message
}
"#;

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

    let server = Command::new(env!("CARGO_BIN_EXE_melodium"))
        .args(["run", "--debug-level", "detailed", "--debug"])
        .arg(&server_debug)
        .arg(file("server.mel", SERVER))
        .args(["--port", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));

    let mut client = Command::new(env!("CARGO_BIN_EXE_melodium"));
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
    assert!(Command::new("kill")
        .args(["-INT", &server.id().to_string()])
        .status()
        .unwrap()
        .success());
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

#[test]
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

#[test]
fn unresolved_secret_headers_fail_the_request() {
    let (server, _, client, _) = exchange(62711, CLIENT, None);
    assert!(
        client.contains("request: secret header 'Authorization': resolution failed"),
        "{}",
        client
    );
    assert!(!server.contains("authorization:"), "{}", server);
}

#[test]
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
