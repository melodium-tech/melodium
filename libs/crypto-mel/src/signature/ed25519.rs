//! Ed25519 signatures of data, such as those of webhooks.

use crate::encoding::{check_encoding, decode};
use ed25519_dalek::pkcs8::DecodePublicKey;
use ed25519_dalek::{Signature, VerifyingKey};
use melodium_core::*;
use melodium_macro::mel_treatment;

/// Gives the key from `public_key`, a PEM public key or the 32 bytes of the key encoded
/// with `encoding`.
pub(crate) fn verifying_key(public_key: &str, encoding: &str) -> Result<VerifyingKey, String> {
    let public_key = public_key.trim();
    if public_key.starts_with("-----BEGIN") {
        VerifyingKey::from_public_key_pem(public_key)
            .map_err(|_| "the public key is not an Ed25519 PEM public key".to_string())
    } else {
        decode(public_key, encoding)
            .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
            .and_then(|bytes| VerifyingKey::from_bytes(&bytes).ok())
            .ok_or_else(|| {
                format!("the public key is neither a PEM public key nor 32 bytes in {encoding}")
            })
    }
}

/// Tells whether `signature` is the Ed25519 signature of `message` by `key`, refusing
/// signatures that are not in their canonical form.
pub(crate) fn verified(key: &VerifyingKey, message: &[u8], signature: &[u8]) -> bool {
    Signature::from_slice(signature)
        .map(|signature| key.verify_strict(message, &signature).is_ok())
        .unwrap_or(false)
}

/// Verifies the Ed25519 signature of data.
///
/// The bytes received through `data` are checked against `signature` with `public_key`,
/// once `data` ends, such as the `X-Signature-Timestamp` header followed by the body of
/// Discord interactions, signed in the `X-Signature-Ed25519` header. `public_key` is a PEM
/// public key (`-----BEGIN PUBLIC KEY-----`), or the 32 bytes of the key encoded with
/// `encoding`, as `signature` is (`hex`, in any case, `base64`, or `base64url`, with or
/// without padding). The data is kept until it ends, as Ed25519 signs messages whole.
///
/// `valid` is emitted if `signature` matches, `invalid` otherwise, including when
/// `signature` is none or is not validly encoded.
/// If `public_key` or `encoding` is not valid, `failed` is emitted and `error` contains
/// the reason.
#[mel_treatment(
    input data Stream<byte>
    input signature Block<Option<string>>
    output valid Block<void>
    output invalid Block<void>
    output failed Block<void>
    output error Block<string>
    default encoding "hex"
)]
pub async fn verify(public_key: string, encoding: string) {
    let key = check_encoding(&encoding).and_then(|_| verifying_key(&public_key, &encoding));
    match key {
        Ok(key) => {
            let mut message = Vec::new();
            while let Ok(bytes) = data.recv_many_as::<u8>().await {
                message.extend_from_slice(&bytes);
            }
            let matching = signature
                .recv_one_as::<Option<String>>()
                .await
                .ok()
                .flatten()
                .and_then(|received| decode(&received, &encoding))
                .map(|received| verified(&key, &message, &received))
                .unwrap_or(false);
            if matching {
                let _ = valid.send_one_as(()).await;
            } else {
                let _ = invalid.send_one_as(()).await;
            }
        }
        Err(message) => {
            let _ = failed.send_one_as(()).await;
            let _ = error.send_one_as(message).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::encode;
    use ed25519_dalek::pkcs8::{spki::der::pem::LineEnding, EncodePublicKey};
    use ed25519_dalek::{Signer, SigningKey};

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7; 32])
    }

    #[test]
    fn signatures_are_verified() {
        let key = signing_key();
        let signature = key.sign(b"1700000000{\"type\":1}").to_bytes();
        let public = key.verifying_key();
        assert!(verified(&public, b"1700000000{\"type\":1}", &signature));
        assert!(!verified(&public, b"1700000001{\"type\":1}", &signature));
        assert!(!verified(
            &public,
            b"1700000000{\"type\":1}",
            &signature[..63]
        ));
    }

    #[test]
    fn public_keys_are_read_as_pem_or_encoded_bytes() {
        let public = signing_key().verifying_key();
        let pem = public.to_public_key_pem(LineEnding::LF).unwrap();
        let hex = encode(public.as_bytes(), "hex");
        let base64 = encode(public.as_bytes(), "base64");
        assert_eq!(verifying_key(&pem, "hex"), Ok(public));
        assert_eq!(verifying_key(&hex, "hex"), Ok(public));
        assert_eq!(verifying_key(&base64, "base64"), Ok(public));
        assert!(verifying_key(&hex[2..], "hex").is_err());
        assert!(verifying_key(
            "-----BEGIN PUBLIC KEY-----\n-----END PUBLIC KEY-----",
            "hex"
        )
        .is_err());
    }
}
