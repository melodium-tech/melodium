//! Random values from the operating system.

use crate::encoding::{check_encoding, encode};
use melodium_core::common::{
    descriptor::DataType,
    executive::{
        Secret as ExecutiveSecret, SecretOrigin, SecretPolicy, SecretReveal, SecretTransmission,
    },
};
use melodium_core::*;
use melodium_macro::mel_treatment;
use zeroize::Zeroizing;

/// Largest number of bytes of a token.
const MAX_BYTES: u64 = 4096;

/// Gives `count` random bytes from the operating system.
#[cfg(feature = "real")]
fn random_bytes(count: usize) -> Result<Zeroizing<Vec<u8>>, String> {
    let mut bytes = Zeroizing::new(vec![0; count]);
    getrandom::fill(&mut bytes)
        .map_err(|_| "the operating system gave no random bytes".to_string())?;
    Ok(bytes)
}

#[cfg(feature = "mock")]
fn random_bytes(_count: usize) -> Result<Zeroizing<Vec<u8>>, String> {
    Err("random bytes are not available in mock mode".to_string())
}

/// Makes a secret named `name` holding `bytes` random bytes encoded with `encoding`.
fn random_token(
    bytes: u64,
    encoding: &str,
    name: String,
    transmission: &str,
    plain_reveal: bool,
) -> Result<ExecutiveSecret, String> {
    check_encoding(encoding)?;
    if bytes == 0 || bytes > MAX_BYTES {
        return Err(format!("bytes must be from 1 to {MAX_BYTES}"));
    }
    let transmission = transmission
        .parse::<SecretTransmission>()
        .map_err(|error| error.to_string())?;
    let random = random_bytes(bytes as usize)?;
    ExecutiveSecret::new(
        name,
        DataType::String,
        SecretPolicy {
            transmission,
            reveal: SecretReveal::Any,
            plain_reveal,
        },
        SecretOrigin::Inline(Value::String(encode(&random, encoding))),
    )
    .map_err(|error| error.to_string())
}

/// Generates a random token, as a secret.
///
/// Once `trigger` is received, `bytes` random bytes from the operating system are encoded
/// with `encoding` (`hex`, `base64`, or `base64url`, without padding) into a token given
/// through `token`, such as a password, a key for webhooks, or a token to give to a client.
/// The secret is named `name`, and its policy is given by `transmission` (`local`,
/// `reference` or `value`) and `plain_reveal` (whether the plain `std/secret::reveal`
/// treatment may reveal it), as `std/secret::conceal` takes.
///
/// If `bytes` is not from 1 to 4096, or `encoding` or `transmission` is not valid,
/// `failed` is emitted and `error` contains the reason.
#[mel_treatment(
    input trigger Block<void>
    output token Block<Secret<string>>
    output failed Block<void>
    output error Block<string>
    default bytes 32
    default encoding "base64url"
    default transmission "local"
    default plain_reveal false
)]
pub async fn token(
    bytes: u64,
    encoding: string,
    name: string,
    transmission: string,
    plain_reveal: bool,
) {
    if trigger.recv_one().await.is_ok() {
        match random_token(bytes, &encoding, name, &transmission, plain_reveal) {
            Ok(secret) => {
                let _ = token.send_one(Value::Secret(secret)).await;
            }
            Err(message) => {
                let _ = failed.send_one_as(()).await;
                let _ = error.send_one_as(message).await;
            }
        }
    }
}

#[cfg(all(test, feature = "real"))]
mod tests {
    use super::*;

    #[test]
    fn tokens_have_the_requested_size_and_policy() {
        let token = random_token(32, "base64url", "csrf".to_string(), "local", true).unwrap();
        assert_eq!(token.name(), "csrf");
        assert!(token.policy().plain_reveal);
        assert_eq!(token.policy().transmission, SecretTransmission::Local);
        assert_eq!(token.datatype(), &DataType::String);

        let first = random_bytes(32).unwrap();
        let second = random_bytes(32).unwrap();
        assert_eq!(first.len(), 32);
        assert_ne!(first, second);
        // 32 bytes are 43 characters in base64url without padding, 64 in hexadecimal.
        assert_eq!(encode(&first, "base64url").len(), 43);
        assert_eq!(encode(&first, "hex").len(), 64);
    }

    #[test]
    fn invalid_settings_are_refused() {
        assert!(random_token(0, "hex", "zero".to_string(), "local", false).is_err());
        assert!(random_token(4097, "hex", "large".to_string(), "local", false).is_err());
        assert!(random_token(32, "base32", "base32".to_string(), "local", false).is_err());
        assert!(random_token(32, "hex", "remote".to_string(), "remote", false).is_err());
    }
}
