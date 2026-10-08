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

use melodium_core::common::descriptor::DataType;
use melodium_core::common::executive::{
    Secret as ExecutiveSecret, SecretOrigin, SecretPolicy, SecretReveal, SecretTransmission,
};
use melodium_core::*;
use melodium_macro::{mel_function, mel_treatment};

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
    secrets_access
)]
pub async fn reveal() {
    if let Ok(Value::Secret(concealed)) = secret.recv_one().await {
        match secrets_access
            .reveal_plainly(&concealed, |content| content.clone())
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
