//! The `api:` source reads secrets from a mock of the API, with the token of the run, and
//! keeps them until wiped. The `api-id:` source requests identity tokens, kept until shortly
//! before they expire.
#![cfg(feature = "real")]

use melodium_core::common::descriptor::DataType;
use melodium_core::common::executive::{PackedArray, SecretSource, Value, Wipe};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use work_mel::secret::{ApiIdentitySource, ApiSource};

const RUN_ID: &str = "12121212-1212-1212-1212-121212121212";
const TOKEN: &str = "cad_v_run-token";

/// Request received by the mock: method and path, then authorization.
type Received = (String, Option<String>);

/// Starts a mock of the API giving secrets, giving its address and the requests it receives.
fn api_server() -> (String, Arc<Mutex<Vec<Received>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let received = Arc::new(Mutex::new(Vec::new()));

    let requests = Arc::clone(&received);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            let mut authorization = None;
            loop {
                let mut line = String::new();
                let _ = reader.read_line(&mut line);
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                if let Some((name, value)) = line.split_once(':') {
                    if name.eq_ignore_ascii_case("authorization") {
                        authorization = Some(value.trim().to_string());
                    }
                }
            }
            let mut parts = request_line.split_whitespace();
            let target = format!(
                "{} {}",
                parts.next().unwrap_or_default(),
                parts.next().unwrap_or_default()
            );
            let name = target
                .strip_prefix(&format!("GET /execution/run/{RUN_ID}/secret/"))
                .unwrap_or_default();
            let audience =
                target.strip_prefix(&format!("GET /execution/run/{RUN_ID}/id-token?audience="));
            let (status, body): (&str, Vec<u8>) = match (audience, name) {
                (Some("vault"), _) => ("200 OK", jwt(600).into_bytes()),
                (Some("short"), _) => ("200 OK", jwt(5).into_bytes()),
                (Some("opaque"), _) => ("200 OK", b"opaque-token\n".to_vec()),
                (Some(_), _) => (
                    "403 Forbidden",
                    b"Secret refused: public runs get no identity token.".to_vec(),
                ),
                (None, "prod/db-password") => ("200 OK", b"db-password-sentinel".to_vec()),
                (None, "binary") => ("200 OK", vec![0xff, 0x00, 0xfe]),
                (None, "forbidden") => (
                    "403 Forbidden",
                    b"Secret refused: public runs cannot read `forbidden`.".to_vec(),
                ),
                _ => (
                    "404 Not Found",
                    b"No secret named `unknown` in this organization.".to_vec(),
                ),
            };
            requests.lock().unwrap().push((target, authorization));
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(&body);
        }
    });

    (address, received)
}

/// Gives a JSON Web Token expiring in `seconds`.
fn jwt(seconds: u64) -> String {
    use base64::Engine;

    let encode = |json: String| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!(
        "{}.{}.signature",
        encode(r#"{"alg":"RS256"}"#.to_string()),
        encode(format!(r#"{{"aud":"vault","exp":{}}}"#, now + seconds))
    )
}

fn source(address: &str) -> Arc<ApiSource> {
    ApiSource::new(
        address.to_string(),
        TOKEN.to_string(),
        Uuid::parse_str(RUN_ID).unwrap(),
    )
}

fn bytes() -> DataType {
    DataType::Vec(Box::new(DataType::Byte))
}

#[test]
fn secrets_are_read_with_the_token_of_the_run() {
    let (address, requests) = api_server();
    let source = source(&address);

    assert_eq!(
        smol::block_on(source.resolve("prod/db-password", &DataType::String)),
        Ok(Value::String("db-password-sentinel".to_string()))
    );
    assert_eq!(
        *requests.lock().unwrap(),
        vec![(
            format!("GET /execution/run/{RUN_ID}/secret/prod/db-password"),
            Some(format!("Bearer {TOKEN}"))
        )]
    );
    assert!(!source.plain_reveal());
    assert!(!format!("{source:?}").contains(TOKEN));
}

#[test]
fn values_are_kept_until_wiped() {
    let (address, requests) = api_server();
    let source = source(&address);
    let resolve = || smol::block_on(source.resolve("prod/db-password", &DataType::String));

    // Checked, then revealed: read once.
    assert!(resolve().is_ok());
    assert!(resolve().is_ok());
    assert_eq!(requests.lock().unwrap().len(), 1);
    assert_eq!(
        smol::block_on(source.resolve("prod/db-password", &bytes())),
        Ok(Value::Packed(PackedArray::Byte(Arc::new(
            b"db-password-sentinel".to_vec()
        ))))
    );
    assert_eq!(requests.lock().unwrap().len(), 1);

    assert!(source.wipe());
    assert!(!source.wipe());
    assert!(resolve().is_ok());
    assert_eq!(requests.lock().unwrap().len(), 2);

    // Ending the engine wipes them too.
    source.shutdown();
    assert!(!source.wipe());
}

#[test]
fn strings_must_be_valid_utf8() {
    let (address, _) = api_server();
    let source = source(&address);

    assert_eq!(
        smol::block_on(source.resolve("binary", &bytes())),
        Ok(Value::Packed(PackedArray::Byte(Arc::new(vec![
            0xff, 0x00, 0xfe
        ]))))
    );
    assert_eq!(
        smol::block_on(source.resolve("binary", &DataType::String)),
        Err("secret 'binary' is not valid UTF-8, it can only be given as Vec<byte>".to_string())
    );
    assert_eq!(
        smol::block_on(source.resolve("binary", &DataType::U64)),
        Err("secret 'binary' cannot be given as u64".to_string())
    );
}

#[test]
fn refusals_give_the_reason_of_the_api() {
    let (address, _) = api_server();
    let source = source(&address);

    assert_eq!(
        smol::block_on(source.resolve("unknown", &DataType::String)),
        Err(
            "Cadence.CI answered 404 Not Found: No secret named `unknown` in this organization."
                .to_string()
        )
    );
    assert_eq!(
        smol::block_on(source.resolve("forbidden", &DataType::String)),
        Err("Cadence.CI answered 403 Forbidden: Secret refused: public runs cannot read `forbidden`.".to_string())
    );
}

#[test]
fn names_cannot_reach_other_endpoints() {
    let (address, requests) = api_server();
    let source = source(&address);

    for name in [
        "",
        "../access",
        "prod/../../access",
        "prod//db",
        "prod/db?x",
        "prod db",
    ] {
        assert_eq!(
            smol::block_on(source.resolve(name, &DataType::String)),
            Err(format!("'{name}' is not a secret name of Cadence.CI, made of letters, digits, '_', '.' and '-', in parts separated by '/'"))
        );
    }
    assert!(requests.lock().unwrap().is_empty());
}

#[test]
fn identity_tokens_are_kept_until_shortly_before_they_expire() {
    let (address, requests) = api_server();
    let source = source(&address);
    let identity = ApiIdentitySource::new(&source);
    let resolve = |audience| smol::block_on(identity.resolve(audience, &DataType::String));

    let token = resolve("vault").unwrap();
    assert!(matches!(&token, Value::String(token) if token.ends_with(".signature")));
    assert_eq!(resolve("vault"), Ok(token));
    assert_eq!(
        *requests.lock().unwrap(),
        vec![(
            format!("GET /execution/run/{RUN_ID}/id-token?audience=vault"),
            Some(format!("Bearer {TOKEN}"))
        )]
    );

    // Expiring within moments, or without expiry to read: requested each time.
    assert!(resolve("short").is_ok());
    assert!(resolve("short").is_ok());
    assert_eq!(
        resolve("opaque"),
        Ok(Value::String("opaque-token".to_string()))
    );
    assert!(resolve("opaque").is_ok());
    assert_eq!(requests.lock().unwrap().len(), 5);

    // Wiped with the secrets of the source.
    assert!(source.wipe());
    assert!(resolve("vault").is_ok());
    assert_eq!(requests.lock().unwrap().len(), 6);
    identity.shutdown();
    assert!(!source.wipe());
    assert!(!identity.plain_reveal());
}

#[test]
fn identity_tokens_need_an_audience_and_a_run_allowed_to() {
    let (address, requests) = api_server();
    let identity = ApiIdentitySource::new(&source(&address));

    assert_eq!(
        smol::block_on(identity.resolve("", &DataType::String)),
        Err("identity tokens need an audience, as in 'api-id:vault'".to_string())
    );
    assert_eq!(
        smol::block_on(identity.resolve("vault", &DataType::U64)),
        Err("identity tokens are given as string, not as u64".to_string())
    );
    assert!(requests.lock().unwrap().is_empty());

    assert_eq!(
        smol::block_on(identity.resolve("https://sts.example.com/a b", &DataType::String)),
        Err(
            "Cadence.CI answered 403 Forbidden: Secret refused: public runs get no identity token."
                .to_string()
        )
    );
    assert_eq!(
        requests.lock().unwrap()[0].0,
        format!(
            "GET /execution/run/{RUN_ID}/id-token?audience=https%3A%2F%2Fsts.example.com%2Fa+b"
        )
    );
}

#[test]
fn secrets_are_refused_outside_cadence_ci() {
    std::env::remove_var("MELODIUM_API_URL");
    std::env::remove_var("MELODIUM_API_TOKEN");
    std::env::set_var("MELODIUM_RUN_ID", RUN_ID);
    assert_eq!(
        smol::block_on(ApiSource::from_environment().resolve("prod/db-password", &DataType::String)),
        Err("'api:' secrets are only given to programs run by Cadence.CI, and MELODIUM_API_URL, MELODIUM_API_TOKEN are not set".to_string())
    );
    assert_eq!(
        smol::block_on(
            ApiIdentitySource::new(&ApiSource::from_environment())
                .resolve("vault", &DataType::String)
        ),
        Err("'api-id:' tokens are only given to programs run by Cadence.CI, and MELODIUM_API_URL, MELODIUM_API_TOKEN are not set".to_string())
    );

    std::env::set_var("MELODIUM_API_URL", "http://127.0.0.1:1");
    std::env::set_var("MELODIUM_API_TOKEN", TOKEN);
    std::env::set_var("MELODIUM_RUN_ID", "not-a-run");
    assert_eq!(
        smol::block_on(
            ApiSource::from_environment().resolve("prod/db-password", &DataType::String)
        ),
        Err("'api:' secrets are only given to programs run by Cadence.CI, and MELODIUM_RUN_ID is not a run identifier".to_string())
    );
}
