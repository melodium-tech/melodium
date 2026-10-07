//! Secrets: creating them from locators, narrowing their policy, concealing and revealing values.
//!
//! A `Secret<T>` holds where its value comes from (such as `env:MELODIUM_SECRET_NAME` or
//! `file:PATH`) rather than the value itself, and is resolved only when an element reveals it.
//! Programs only get the environment variables whose name starts with `MELODIUM_SECRET_`.
//! Every reveal is checked against the secret policy and recorded as a debug event,
//! and refused or failed ones are also written to the log.
//!
//! The policy of a secret is set when it is created, and can only be narrowed afterwards.
//! By default, a secret stays on the engine that holds it, any element may reveal it,
//! and the plain `reveal` treatment may not. Secrets from the environment and from files
//! are never plainly revealed, whatever their policy.
//!
//! Secrets can be derived from other secrets (formatted, encoded) without revealing them:
//! a derived secret is computed when revealed, and gets the most restrictive combination
//! of the policies of the secrets it comes from.
//!
//! When a program is distributed, secrets given to a distant engine follow their
//! transmission: `local` ones are refused, `reference` ones send only their locator,
//! resolved by the distant engine with its own sources (derived secrets have none),
//! and `value` ones send their value, over TLS only. The distant engine keeps the policy.
//!
//! Plaintext held by Mélodium (values of concealed secrets, revealed values, values masked
//! in logs, vault caches) is overwritten once not needed anymore, and before the process
//! exits, including on SIGINT, SIGTERM and SIGHUP. This is best effort, and does not cover:
//! - a process stopped by SIGKILL, the out-of-memory killer or a power loss;
//! - memory written to swap or to core dumps;
//! - copies made by libraries, child processes and remote services given the value;
//! - buffers of messages carrying a secret sent by value to a distant engine,
//! nor values made plain by `reveal`, which become ordinary data.

use crate::data::map::*;
use base64::Engine;
use melodium_core::common::descriptor::DataType;
use melodium_core::common::executive::{
    Secret as ExecutiveSecret, SecretDerivation, SecretOrigin, SecretPolicy, SecretReveal,
    SecretTransmission,
};
use melodium_core::*;
use melodium_macro::{mel_function, mel_treatment};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use std::collections::HashMap;
use std::sync::Arc;
use zeroize::Zeroize;

/// Secret read from the environment variable `variable` when revealed.
///
/// Only variables whose name starts with `MELODIUM_SECRET_` are given to programs.
#[mel_function]
pub fn from_environment(variable: string, name: string) -> Secret<string> {
    ExecutiveSecret::new(
        name,
        DataType::String,
        SecretPolicy::default(),
        SecretOrigin::Locator(format!("env:{variable}")),
    )
    .unwrap()
}

/// Secret read from the file at `path` when revealed.
///
/// One trailing newline is removed from the content.
#[mel_function]
pub fn from_file(path: string, name: string) -> Secret<string> {
    ExecutiveSecret::new(
        name,
        DataType::String,
        SecretPolicy::default(),
        SecretOrigin::Locator(format!("file:{path}")),
    )
    .unwrap()
}

/// Secret read from the file at `path` when revealed, as bytes.
///
/// The content is kept unchanged.
#[mel_function]
pub fn from_file_bytes(path: string, name: string) -> Secret<Vec<byte>> {
    ExecutiveSecret::new(
        name,
        DataType::Vec(Box::new(DataType::Byte)),
        SecretPolicy::default(),
        SecretOrigin::Locator(format!("file:{path}")),
    )
    .unwrap()
}

/// Secret resolved from `locator` when revealed, with an explicit policy.
///
/// - `locator`: `<scheme>:<path>`, such as `env:MELODIUM_SECRET_NAME`, `file:PATH`, or a scheme registered by a source model.
/// - `transmission`: where the secret may go when the program is distributed,
/// `local` (never leaves this engine), `reference` (only the locator is sent),
/// or `value` (the value is sent).
/// - `plain_reveal`: whether the plain `reveal` treatment may reveal it, never for
///   `env:` and `file:` secrets.
///
/// Gives none if `locator` or `transmission` are not valid.
#[mel_function(
    generic T ()
)]
pub fn locate(
    locator: string,
    name: string,
    transmission: string,
    plain_reveal: bool,
) -> Option<Secret<T>> {
    transmission
        .parse::<SecretTransmission>()
        .and_then(|transmission| {
            ExecutiveSecret::new(
                name,
                generics.get("T").unwrap().clone(),
                SecretPolicy {
                    transmission,
                    reveal: SecretReveal::Any,
                    plain_reveal,
                },
                SecretOrigin::Locator(locator),
            )
        })
        .ok()
}

/// Gives the name of a secret.
#[mel_function(
    generic T ()
)]
pub fn name(secret: Secret<T>) -> string {
    secret.name().to_string()
}

/// Keeps a secret on the engine that holds it, it is never sent to another one.
#[mel_function(
    generic T ()
)]
pub fn keep_local(secret: Secret<T>) -> Secret<T> {
    secret.narrow(&SecretPolicy {
        transmission: SecretTransmission::Local,
        ..SecretPolicy::unrestricted()
    })
}

/// Limits a secret to be sent to other engines by reference only, its value is never sent.
#[mel_function(
    generic T ()
)]
pub fn limit_to_reference(secret: Secret<T>) -> Secret<T> {
    secret.narrow(&SecretPolicy {
        transmission: SecretTransmission::Reference,
        ..SecretPolicy::unrestricted()
    })
}

/// Limits the elements that may reveal a secret to the ones in `elements`,
/// given as identifiers such as `std/secret::reveal`.
#[mel_function(
    generic T ()
)]
pub fn reveal_only_by(secret: Secret<T>, elements: Vec<string>) -> Secret<T> {
    secret.narrow(&SecretPolicy {
        reveal: SecretReveal::Only(elements),
        ..SecretPolicy::unrestricted()
    })
}

/// Forbids the plain `reveal` treatment to reveal a secret.
#[mel_function(
    generic T ()
)]
pub fn forbid_plain_reveal(secret: Secret<T>) -> Secret<T> {
    secret.narrow(&SecretPolicy {
        plain_reveal: false,
        ..SecretPolicy::unrestricted()
    })
}

/// Gives the bytes of a string or bytes value.
fn value_bytes(value: &Value) -> Option<std::borrow::Cow<'_, [u8]>> {
    match value {
        Value::String(text) => Some(std::borrow::Cow::Borrowed(text.as_bytes())),
        Value::Packed(PackedArray::Byte(bytes)) | Value::Packed(PackedArray::U8(bytes)) => {
            Some(std::borrow::Cow::Borrowed(bytes.as_slice()))
        }
        Value::Vec(values) => values
            .iter()
            .map(|value| match value {
                Value::Byte(byte) | Value::U8(byte) => Some(*byte),
                _ => None,
            })
            .collect::<Option<Vec<u8>>>()
            .map(std::borrow::Cow::Owned),
        _ => None,
    }
}

/// Template filled with plain strings and string secrets.
#[derive(Debug)]
struct FormatDerivation {
    template: String,
    plain_entries: HashMap<String, String>,
    /// Entries given by the input secrets, in their order.
    secret_entries: Vec<String>,
    /// Entries that are neither strings nor string secrets.
    invalid_entries: Vec<String>,
}

impl SecretDerivation for FormatDerivation {
    fn derive(&self, inputs: &[Value]) -> Result<Value, String> {
        if let Some(entry) = self.invalid_entries.first() {
            return Err(format!(
                "entry '{entry}' is neither a string nor a Secret<string>"
            ));
        }
        let mut entries = self.plain_entries.clone();
        let mut invalid = None;
        for (entry, value) in self.secret_entries.iter().zip(inputs) {
            match value {
                Value::String(text) => {
                    entries.insert(entry.clone(), text.clone());
                }
                _ => {
                    invalid = Some(format!("entry '{entry}' is not a string"));
                    break;
                }
            }
        }
        let formatted = match invalid {
            Some(invalid) => Err(invalid),
            None => Ok(strfmt::strfmt(&self.template, &entries)),
        };
        for (_, mut text) in entries.drain() {
            text.zeroize();
        }
        // strfmt errors may quote formatted content, so only their kind is kept.
        formatted?.map(Value::String).map_err(|err| match err {
            strfmt::FmtError::Invalid(_) => "the template is not valid".to_string(),
            strfmt::FmtError::KeyError(_) => {
                "the template uses an entry that is not given".to_string()
            }
            strfmt::FmtError::TypeError(_) => {
                "the template has an invalid placeholder format".to_string()
            }
        })
    }
}

#[derive(Debug)]
struct Base64Derivation;

impl SecretDerivation for Base64Derivation {
    fn derive(&self, inputs: &[Value]) -> Result<Value, String> {
        let bytes = inputs
            .first()
            .and_then(value_bytes)
            .ok_or_else(|| "base64 needs a string or bytes".to_string())?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
        if let std::borrow::Cow::Owned(mut bytes) = bytes {
            bytes.zeroize();
        }
        Ok(Value::String(encoded))
    }
}

/// Every character except the unreserved ones of RFC 3986 (`A-Z a-z 0-9 - . _ ~`).
const URL_COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

#[derive(Debug)]
struct UrlEncodeDerivation;

impl SecretDerivation for UrlEncodeDerivation {
    fn derive(&self, inputs: &[Value]) -> Result<Value, String> {
        match inputs.first() {
            Some(Value::String(text)) => Ok(Value::String(
                utf8_percent_encode(text, URL_COMPONENT).to_string(),
            )),
            _ => Err("URL encoding needs a string".to_string()),
        }
    }
}

#[derive(Debug)]
struct BytesDerivation;

impl SecretDerivation for BytesDerivation {
    fn derive(&self, inputs: &[Value]) -> Result<Value, String> {
        match inputs.first() {
            Some(Value::String(text)) => Ok(Value::Packed(PackedArray::Byte(Arc::new(
                text.as_bytes().to_vec(),
            )))),
            _ => Err("bytes conversion needs a string".to_string()),
        }
    }
}

/// Secret made of `template` filled with `entries`, computed when revealed.
///
/// `template` contains braced placeholders, such as `"Bearer {token}"`.
/// `entries` maps placeholders to `string` values or `Secret<string>` values,
/// built with `std/data/map::|map` and `|entry`.
/// The secret is named `name`, and gets the most restrictive combination
/// of the policies of the secrets in `entries`.
///
/// Revealing fails if a placeholder has no entry, or if an entry is neither
/// a `string` nor a `Secret<string>`.
#[mel_function]
pub fn format(template: string, entries: Map, name: string) -> Secret<string> {
    let mut plain_entries = HashMap::new();
    let mut secret_entries = Vec::new();
    let mut invalid_entries = Vec::new();
    let mut inputs = Vec::new();
    for (entry, value) in entries.map {
        match value {
            Value::String(text) => {
                plain_entries.insert(entry, text);
            }
            Value::Secret(secret) if secret.datatype() == &DataType::String => {
                secret_entries.push(entry);
                inputs.push(secret);
            }
            _ => invalid_entries.push(entry),
        }
    }
    ExecutiveSecret::derive(
        name,
        DataType::String,
        inputs,
        Arc::new(FormatDerivation {
            template,
            plain_entries,
            secret_entries,
            invalid_entries,
        }),
    )
    .unwrap()
}

/// Secret made of the base64 encoding (standard, padded) of a string secret, computed when revealed.
///
/// Such as the credentials of HTTP Basic authentication: `|base64(|format("{user}:{password}", ...), ...)`.
#[mel_function]
pub fn base64(secret: Secret<string>, name: string) -> Secret<string> {
    ExecutiveSecret::derive(
        name,
        DataType::String,
        vec![secret],
        Arc::new(Base64Derivation),
    )
    .unwrap()
}

/// Secret made of the base64 encoding (standard, padded) of a bytes secret, computed when revealed.
#[mel_function]
pub fn base64_bytes(secret: Secret<Vec<byte>>, name: string) -> Secret<string> {
    ExecutiveSecret::derive(
        name,
        DataType::String,
        vec![secret],
        Arc::new(Base64Derivation),
    )
    .unwrap()
}

/// Secret made of the URL encoding of a string secret, computed when revealed.
///
/// Every character except `A-Z a-z 0-9 - . _ ~` is percent-encoded,
/// so the result fits in any URL part, such as credentials: `https://{user}:{password}@host`.
#[mel_function]
pub fn url_encode(secret: Secret<string>, name: string) -> Secret<string> {
    ExecutiveSecret::derive(
        name,
        DataType::String,
        vec![secret],
        Arc::new(UrlEncodeDerivation),
    )
    .unwrap()
}

/// Secret made of the UTF-8 bytes of a string secret, computed when revealed.
#[mel_function]
pub fn to_bytes(secret: Secret<string>, name: string) -> Secret<Vec<byte>> {
    ExecutiveSecret::derive(
        name,
        DataType::Vec(Box::new(DataType::Byte)),
        vec![secret],
        Arc::new(BytesDerivation),
    )
    .unwrap()
}

/// Conceals a value into a secret.
///
/// Intended for values only known at runtime, such as a token given by an API response.
/// The secret is named `name`, its policy is given by `transmission` (`local`, `reference` or `value`)
/// and `plain_reveal` (whether the plain `reveal` treatment may reveal it).
///
/// If the secret cannot be created, `failed` is emitted and `error` contains the reason.
#[mel_treatment(
    generic T ()
    input value Block<T>
    output secret Block<Secret<T>>
    output failed Block<void>
    output error Block<string>
    default transmission "local"
    default plain_reveal false
)]
pub async fn conceal(name: string, transmission: string, plain_reveal: bool) {
    if let Ok(content) = value.recv_one().await {
        let concealed = transmission
            .parse::<SecretTransmission>()
            .and_then(|transmission| {
                ExecutiveSecret::new(
                    name,
                    T,
                    SecretPolicy {
                        transmission,
                        reveal: SecretReveal::Any,
                        plain_reveal,
                    },
                    SecretOrigin::Inline(content),
                )
            });
        match concealed {
            Ok(concealed) => {
                let _ = secret.send_one(Value::Secret(concealed)).await;
            }
            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(err.to_string().into()).await;
            }
        }
    }
}

/// Reveals the value of a secret, as a plain value.
///
/// The secret policy must allow plain reveal, which is not the default. Secrets from the
/// environment and from files (`env:`, `file:`) are never plainly revealed.
/// The access is recorded as a debug event, and a refused or failed one is also written to the log.
///
/// If the value cannot be revealed, `failed` is emitted and `error` contains the reason.
#[mel_treatment(
    generic T ()
    input secret Block<Secret<T>>
    output value Block<T>
    output failed Block<void>
    output error Block<string>
)]
pub async fn reveal() {
    if let Ok(Value::Secret(concealed)) = secret.recv_one().await {
        match concealed
            .reveal_plainly(&secret_access, |content| content.clone())
            .await
        {
            Ok(content) => {
                let _ = value.send_one(content).await;
            }
            Err(err) => {
                let _ = failed.send_one(().into()).await;
                let _ = error.send_one(err.to_string().into()).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: &str = "s3cr3t:value/@";

    #[test]
    fn format_fills_plain_and_secret_entries() {
        let derivation = FormatDerivation {
            template: "{scheme}://{user}:{password}@host".to_string(),
            plain_entries: HashMap::from([
                ("scheme".to_string(), "https".to_string()),
                ("user".to_string(), "ci".to_string()),
            ]),
            secret_entries: vec!["password".to_string()],
            invalid_entries: Vec::new(),
        };
        assert_eq!(
            derivation.derive(&[Value::String(SENTINEL.to_string())]),
            Ok(Value::String(format!("https://ci:{SENTINEL}@host")))
        );
    }

    #[test]
    fn format_errors_never_show_values() {
        let missing = FormatDerivation {
            template: "{missing} {password}".to_string(),
            plain_entries: HashMap::new(),
            secret_entries: vec!["password".to_string()],
            invalid_entries: Vec::new(),
        };
        let invalid_format = FormatDerivation {
            template: "{password:>.3e}".to_string(),
            plain_entries: HashMap::new(),
            secret_entries: vec!["password".to_string()],
            invalid_entries: Vec::new(),
        };
        let invalid_entry = FormatDerivation {
            template: "{password}".to_string(),
            plain_entries: HashMap::new(),
            secret_entries: vec!["password".to_string()],
            invalid_entries: vec!["count".to_string()],
        };
        for derivation in [missing, invalid_format, invalid_entry] {
            let error = derivation
                .derive(&[Value::String(SENTINEL.to_string())])
                .unwrap_err();
            assert!(!error.contains(SENTINEL), "leaked in {error}");
        }
    }

    #[test]
    fn encodings() {
        let value = Value::String(SENTINEL.to_string());
        assert_eq!(
            UrlEncodeDerivation.derive(&[value.clone()]),
            Ok(Value::String("s3cr3t%3Avalue%2F%40".to_string()))
        );
        assert_eq!(
            UrlEncodeDerivation.derive(&[Value::String("aZ09-._~ é".to_string())]),
            Ok(Value::String("aZ09-._~%20%C3%A9".to_string()))
        );
        assert_eq!(
            Base64Derivation.derive(&[Value::String("ci:password".to_string())]),
            Ok(Value::String("Y2k6cGFzc3dvcmQ=".to_string()))
        );
        let bytes = BytesDerivation.derive(&[value]).unwrap();
        assert_eq!(
            bytes,
            Value::Packed(PackedArray::Byte(Arc::new(SENTINEL.as_bytes().to_vec())))
        );
        assert_eq!(bytes.datatype(), DataType::Vec(Box::new(DataType::Byte)));
        assert_eq!(
            Base64Derivation.derive(&[Value::Vec(vec![Value::Byte(0xff), Value::Byte(0)])]),
            Ok(Value::String("/wA=".to_string()))
        );
        assert!(Base64Derivation.derive(&[Value::U64(1)]).is_err());
    }
}
