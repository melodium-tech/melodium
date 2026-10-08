//! Secrets received at runtime, derived and narrowed without being revealed.
//!
//! These treatments do what the functions of `std/secret` do, for secrets arriving on
//! inputs, such as those given by `std/secret::conceal` or a model reading them at runtime.

use super::*;

/// Secret made of `template` filled with `entries`, computed when revealed.
///
/// Same as `std/secret::|format`, for entries received at runtime, built with the
/// treatments of `std/data/map/block`. `entries` maps placeholders to `string` values or
/// `Secret<string>` values. The secret is named `name`, and gets the most restrictive
/// combination of the policies of the secrets in `entries`.
///
/// Revealing fails if a placeholder has no entry, or if an entry is neither
/// a `string` nor a `Secret<string>`.
#[mel_treatment(
    input entries Block<Map>
    output secret Block<Secret<string>>
)]
pub async fn format(template: string, name: string) {
    if let Ok(entries) = entries.recv_one_as::<Arc<Map>>().await {
        let formatted = formatted(template, Arc::unwrap_or_clone(entries).map, name);
        let _ = secret.send_one(Value::Secret(formatted)).await;
    }
}

/// Secret made of the base64 encoding (standard, padded) of a string secret,
/// computed when revealed.
///
/// Same as `std/secret::|base64`, for a secret received at runtime.
#[mel_treatment(
    input secret Block<Secret<string>>
    output encoded Block<Secret<string>>
)]
pub async fn base64(name: string) {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        let derived = derived(name, DataType::String, received, Arc::new(Base64Derivation));
        let _ = encoded.send_one(Value::Secret(derived)).await;
    }
}

/// Secret made of the base64 encoding (standard, padded) of a bytes secret,
/// computed when revealed.
///
/// Same as `std/secret::|base64_bytes`, for a secret received at runtime.
#[mel_treatment(
    input secret Block<Secret<Vec<byte>>>
    output encoded Block<Secret<string>>
)]
pub async fn base64_bytes(name: string) {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        let derived = derived(name, DataType::String, received, Arc::new(Base64Derivation));
        let _ = encoded.send_one(Value::Secret(derived)).await;
    }
}

/// Secret made of the URL encoding of a string secret, computed when revealed.
///
/// Same as `std/secret::|url_encode`, for a secret received at runtime.
#[mel_treatment(
    input secret Block<Secret<string>>
    output encoded Block<Secret<string>>
)]
pub async fn url_encode(name: string) {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        let derived = derived(
            name,
            DataType::String,
            received,
            Arc::new(UrlEncodeDerivation),
        );
        let _ = encoded.send_one(Value::Secret(derived)).await;
    }
}

/// Secret made of the UTF-8 bytes of a string secret, computed when revealed.
///
/// Same as `std/secret::|to_bytes`, for a secret received at runtime.
#[mel_treatment(
    input secret Block<Secret<string>>
    output bytes Block<Secret<Vec<byte>>>
)]
pub async fn to_bytes(name: string) {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        let derived = derived(
            name,
            DataType::Vec(Box::new(DataType::Byte)),
            received,
            Arc::new(BytesDerivation),
        );
        let _ = bytes.send_one(Value::Secret(derived)).await;
    }
}

/// Keeps a secret received at runtime on the engine that holds it, it is never sent
/// to another one.
///
/// Same as `std/secret::|keep_local`.
#[mel_treatment(
    generic T ()
    input secret Block<Secret<T>>
    output narrowed Block<Secret<T>>
)]
pub async fn keep_local() {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        let _ = narrowed
            .send_one(Value::Secret(received.narrow(&keep_local_narrowing())))
            .await;
    }
}

/// Limits a secret received at runtime to be sent to other engines by reference only,
/// its value is never sent.
///
/// Same as `std/secret::|limit_to_reference`.
#[mel_treatment(
    generic T ()
    input secret Block<Secret<T>>
    output narrowed Block<Secret<T>>
)]
pub async fn limit_to_reference() {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        let _ = narrowed
            .send_one(Value::Secret(
                received.narrow(&limit_to_reference_narrowing()),
            ))
            .await;
    }
}

/// Limits the elements that may reveal a secret received at runtime to the ones in
/// `elements`, given as identifiers such as `std/secret::reveal`.
///
/// Same as `std/secret::|reveal_only_by`.
#[mel_treatment(
    generic T ()
    input secret Block<Secret<T>>
    output narrowed Block<Secret<T>>
)]
pub async fn reveal_only_by(elements: Vec<string>) {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        let _ = narrowed
            .send_one(Value::Secret(
                received.narrow(&reveal_only_by_narrowing(elements)),
            ))
            .await;
    }
}

/// Forbids the plain `reveal` treatment to reveal a secret received at runtime.
///
/// Same as `std/secret::|forbid_plain_reveal`.
#[mel_treatment(
    generic T ()
    input secret Block<Secret<T>>
    output narrowed Block<Secret<T>>
)]
pub async fn forbid_plain_reveal() {
    if let Ok(Value::Secret(received)) = secret.recv_one().await {
        let _ = narrowed
            .send_one(Value::Secret(
                received.narrow(&forbid_plain_reveal_narrowing()),
            ))
            .await;
    }
}
