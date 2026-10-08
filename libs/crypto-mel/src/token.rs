//! Tokens received from clients, checked against secrets.

use melodium_core::common::executive::Secret as ExecutiveSecret;
use melodium_core::*;
use melodium_macro::mel_treatment;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

/// Tells, in constant time for a given length, whether `received` equals `expected`.
pub(crate) fn equal(received: &[u8], expected: &[u8]) -> bool {
    received.ct_eq(expected).into()
}

/// Checks a received token against a secret.
///
/// `received` is compared with `token` in constant time, such as the `X-Gitlab-Token`
/// header of GitLab webhooks, given as a secret by an `http/server::HttpServer` listing it in
/// its `secret_headers`. Both are revealed only for the comparison.
///
/// `valid` is emitted if they are equal, `invalid` otherwise, including when `received`
/// is none or is not a `Secret<string>`.
/// If a secret cannot be revealed, `failed` is emitted and `error` contains the reason.
#[mel_treatment(
    input received Block<Option<Secret<string>>>
    output valid Block<void>
    output invalid Block<void>
    output failed Block<void>
    output error Block<string>
    secrets_access
)]
pub async fn check(token: Secret<string>) {
    let received = match received.recv_one().await {
        Ok(Value::Option(Some(value))) => match *value {
            Value::Secret(received) => Some(received),
            _ => None,
        },
        _ => None,
    };
    let Some(received) = received else {
        let _ = invalid.send_one_as(()).await;
        return;
    };
    match compare(&secrets_access, &received, &token).await {
        Ok(true) => {
            let _ = valid.send_one_as(()).await;
        }
        Ok(false) => {
            let _ = invalid.send_one_as(()).await;
        }
        Err(message) => {
            let _ = failed.send_one_as(()).await;
            let _ = error.send_one_as(message).await;
        }
    }
}

async fn compare(
    access: &melodium_core::common::executive::SecretsAccess,
    received: &ExecutiveSecret,
    token: &ExecutiveSecret,
) -> Result<bool, String> {
    let received = access
        .reveal_str(received, |value| Zeroizing::new(value.as_bytes().to_vec()))
        .await
        .map_err(|error| format!("received token: {error}"))?;
    access
        .reveal_str(token, |token| equal(&received, token.as_bytes()))
        .await
        .map_err(|error| format!("token: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_compared() {
        assert!(equal(b"t0ken", b"t0ken"));
        assert!(!equal(b"t0ken", b"t0kem"));
        assert!(!equal(b"t0ken", b"t0ken-longer"));
        assert!(!equal(b"", b"t0ken"));
    }
}
