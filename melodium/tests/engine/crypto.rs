//! The `crypto` package signs data with HMAC, verifies Ed25519 signatures, signs and
//! verifies JSON Web Tokens, and makes random tokens.

use super::common::{self, revealed};
use melodium_common::descriptor::DataType;
use melodium_common::executive::{Secret, SecretOrigin, SecretPolicy, Value};
use std::collections::HashMap;

const SIGN: &str = include_str!("scripts/crypto_sign.mel");
const SIGNATURES: &str = include_str!("scripts/crypto_signatures.mel");

#[test]
fn data_is_signed_with_hmac() {
    std::env::set_var("MELODIUM_SECRET_CRYPTO_TEST_KEY", "Jefe");
    let (logs, _) = common::run(SIGN, HashMap::new());
    let message = |label: &str| common::message(&logs, label).to_string();

    // RFC 4231, test case 2.
    assert_eq!(
        message("hex"),
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
    assert_eq!(
        message("base64"),
        "W9zBRr9gdU5qBCQmCJV1x1oAPwidJzmDnexYuWTsOEM="
    );
    assert_eq!(
        message("sha512"),
        "164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea2505549758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737"
    );
    // RFC 2202, test case 2.
    assert_eq!(message("sha1"), "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79");
    assert_eq!(
        message("base64url"),
        "W9zBRr9gdU5qBCQmCJV1x1oAPwidJzmDnexYuWTsOEM"
    );
    assert!(
        message("unknown").contains("'md5' is not a supported algorithm"),
        "{}",
        message("unknown")
    );
}

/// Ed25519 key, signature and message made with OpenSSL for these tests only.
const ED25519_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIDYgidPLfA5XDmmpZIJQm8gzlQ2z+JRkWcYrCbokgD3Q
-----END PRIVATE KEY-----
";
const ED25519_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAd+fOas/+v316/EwswKbhnpu5Of4/edtkcUsQLNRTMrA=
-----END PUBLIC KEY-----
";
const ED25519_PUBLIC_KEY_HEX: &str =
    "77e7ce6acffebf7d7afc4c2cc0a6e19e9bb939fe3f79db64714b102cd45332b0";
const ED25519_MESSAGE: &str = r#"1700000000{"type":1}"#;
const ED25519_SIGNATURE: &str = "b42215c7babce722ae355eef6703e26055126a9100257ff9cbf02a79787b3fe1025dc0cc1f616d6793a0da5e389707bbdf73d6772f5c0419cb9cae04da698106";

fn inline_secret(name: &str, value: &str, plain_reveal: bool) -> Value {
    Value::Secret(
        Secret::new(
            name.to_string(),
            DataType::String,
            SecretPolicy {
                plain_reveal,
                ..SecretPolicy::default()
            },
            SecretOrigin::Inline(Value::String(value.to_string())),
        )
        .unwrap(),
    )
}

#[test]
fn signatures_tokens_and_random_values() {
    let (logs, events) = common::run(
        SIGNATURES,
        HashMap::from([
            (
                "message".to_string(),
                Value::String(ED25519_MESSAGE.to_string()),
            ),
            (
                "public_key".to_string(),
                Value::String(ED25519_PUBLIC_KEY_HEX.to_string()),
            ),
            (
                "signature".to_string(),
                Value::String(ED25519_SIGNATURE.to_string()),
            ),
            (
                "jwt_key".to_string(),
                inline_secret("jwt_key", ED25519_PRIVATE_KEY, false),
            ),
            (
                "jwt_public_key".to_string(),
                Value::String(ED25519_PUBLIC_KEY.to_string()),
            ),
            (
                "hmac_key".to_string(),
                inline_secret("hmac_key", "shared", true),
            ),
        ]),
    );
    let message = |label: &str| common::message(&logs, label).to_string();

    assert_eq!(message("ed25519-good"), "valid");
    assert_eq!(message("ed25519-wrong"), "invalid");

    assert!(common::log(&logs, "jwt-reason").is_none(), "{:?}", logs);
    assert!(common::log(&logs, "jwt-error").is_none(), "{:?}", logs);
    let claims = message("jwt-claims");
    assert!(claims.contains(r#""iss":"ci""#), "{}", claims);
    assert!(claims.contains(r#""exp":"#), "{}", claims);

    // Header and claims of the token, then its HMAC-SHA256 signature, as computed apart.
    assert_eq!(
        revealed(&events, "revealHmacJwt"),
        vec![Value::String(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJjaSJ9.Hm26zhxUK6vazRYvzgn577oBfG-sTD3DdUZnbiw926A"
                .to_string()
        )]
    );

    match &revealed(&events, "revealToken")[..] {
        [Value::String(token)] => {
            assert_eq!(token.len(), 32, "{}", token);
            assert!(token.chars().all(|c| c.is_ascii_hexdigit()), "{}", token);
        }
        other => panic!("no random token revealed: {:?}", other),
    }
}
