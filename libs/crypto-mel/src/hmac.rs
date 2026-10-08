//! HMAC signatures of data, such as those of webhooks.
//!
//! Keys are secrets, revealed by the treatments when they start. Signatures are encoded as
//! text, in hexadecimal or base64, as services usually give them in headers.

use base64::Engine;
use hmac::{Hmac, Mac};
use melodium_core::*;
use melodium_macro::mel_treatment;
use sha2::{Sha256, Sha384, Sha512};

/// HMAC being computed, with one of the supported hash functions.
pub(crate) enum Signer {
    Sha256(Hmac<Sha256>),
    Sha384(Hmac<Sha384>),
    Sha512(Hmac<Sha512>),
}

impl Signer {
    pub(crate) fn new(algorithm: &str, key: &[u8]) -> Result<Self, String> {
        // HMAC takes keys of any length, so creating one never fails.
        let invalid = |_| "the key cannot be used".to_string();
        match algorithm {
            "sha256" => Ok(Self::Sha256(
                Hmac::<Sha256>::new_from_slice(key).map_err(invalid)?,
            )),
            "sha384" => Ok(Self::Sha384(
                Hmac::<Sha384>::new_from_slice(key).map_err(invalid)?,
            )),
            "sha512" => Ok(Self::Sha512(
                Hmac::<Sha512>::new_from_slice(key).map_err(invalid)?,
            )),
            _ => Err(format!(
                "'{algorithm}' is not a supported algorithm, expected 'sha256', 'sha384' or 'sha512'"
            )),
        }
    }

    pub(crate) fn update(&mut self, data: &[u8]) {
        match self {
            Self::Sha256(mac) => mac.update(data),
            Self::Sha384(mac) => mac.update(data),
            Self::Sha512(mac) => mac.update(data),
        }
    }

    pub(crate) fn sign(self) -> Vec<u8> {
        match self {
            Self::Sha256(mac) => mac.finalize().into_bytes().to_vec(),
            Self::Sha384(mac) => mac.finalize().into_bytes().to_vec(),
            Self::Sha512(mac) => mac.finalize().into_bytes().to_vec(),
        }
    }

    /// Tells, in constant time, whether `signature` is the HMAC of the data.
    pub(crate) fn verify(self, signature: &[u8]) -> bool {
        match self {
            Self::Sha256(mac) => mac.verify_slice(signature).is_ok(),
            Self::Sha384(mac) => mac.verify_slice(signature).is_ok(),
            Self::Sha512(mac) => mac.verify_slice(signature).is_ok(),
        }
    }
}

fn check_encoding(encoding: &str) -> Result<(), String> {
    match encoding {
        "hex" | "base64" => Ok(()),
        _ => Err(format!(
            "'{encoding}' is not a supported encoding, expected 'hex' or 'base64'"
        )),
    }
}

pub(crate) fn encode(bytes: &[u8], encoding: &str) -> String {
    match encoding {
        "base64" => base64::engine::general_purpose::STANDARD.encode(bytes),
        _ => hex::encode(bytes),
    }
}

/// Gives the bytes of `text` encoded as `encoding`, none if it is not valid.
pub(crate) fn decode(text: &str, encoding: &str) -> Option<Vec<u8>> {
    match encoding {
        "base64" => base64::engine::general_purpose::STANDARD.decode(text).ok(),
        _ => hex::decode(text).ok(),
    }
}

/// Makes the signer for `key`, revealed on behalf of `access`.
async fn signer(
    access: &melodium_core::common::executive::SecretsAccess,
    key: &melodium_core::common::executive::Secret,
    algorithm: &str,
    encoding: &str,
) -> Result<Signer, String> {
    check_encoding(encoding)?;
    access
        .reveal_str(key, |key| Signer::new(algorithm, key.as_bytes()))
        .await
        .map_err(|error| format!("key: {error}"))?
}

/// Signs data with HMAC.
///
/// The HMAC of the bytes received through `data` is computed with `key` and `algorithm`
/// (`sha256`, `sha384` or `sha512`), and given through `signature` once `data` ends,
/// encoded with `encoding` (`hex`, in lowercase, or `base64`), such as to sign the body of
/// an outgoing webhook. The key is revealed when the treatment starts.
///
/// If the key cannot be revealed, or `algorithm` or `encoding` is not supported,
/// `failed` is emitted and `error` contains the reason.
#[mel_treatment(
    input data Stream<byte>
    output signature Block<string>
    output failed Block<void>
    output error Block<string>
    default algorithm "sha256"
    default encoding "hex"
    secrets_access
)]
pub async fn sign(key: Secret<string>, algorithm: string, encoding: string) {
    match signer(&secrets_access, &key, &algorithm, &encoding).await {
        Ok(mut signer) => {
            while let Ok(bytes) = data.recv_many_as::<u8>().await {
                signer.update(&bytes);
            }
            let _ = signature
                .send_one_as(encode(&signer.sign(), &encoding))
                .await;
        }
        Err(message) => {
            let _ = failed.send_one_as(()).await;
            let _ = error.send_one_as(message).await;
        }
    }
}

/// Verifies the HMAC signature of data.
///
/// The HMAC of the bytes received through `data` is computed with `key` and `algorithm`
/// (`sha256`, `sha384` or `sha512`), and compared in constant time with `signature`, once
/// `data` ends. `signature` is encoded with `encoding` (`hex`, in any case, or `base64`),
/// and starts with `prefix` when given, such as `sha256=` for GitHub webhooks
/// (`X-Hub-Signature-256` header). The key is revealed when the treatment starts.
///
/// `valid` is emitted if `signature` matches, `invalid` otherwise, including when
/// `signature` is none, lacks `prefix`, or is not validly encoded.
/// If the key cannot be revealed, or `algorithm` or `encoding` is not supported,
/// `failed` is emitted and `error` contains the reason.
#[mel_treatment(
    input data Stream<byte>
    input signature Block<Option<string>>
    output valid Block<void>
    output invalid Block<void>
    output failed Block<void>
    output error Block<string>
    default algorithm "sha256"
    default encoding "hex"
    default prefix ""
    secrets_access
)]
pub async fn verify(key: Secret<string>, algorithm: string, encoding: string, prefix: string) {
    match signer(&secrets_access, &key, &algorithm, &encoding).await {
        Ok(mut signer) => {
            while let Ok(bytes) = data.recv_many_as::<u8>().await {
                signer.update(&bytes);
            }
            let received = signature
                .recv_one_as::<Option<String>>()
                .await
                .ok()
                .flatten();
            let matching = received
                .as_deref()
                .and_then(|received| received.strip_prefix(prefix.as_str()))
                .and_then(|received| decode(received, &encoding))
                .map(|received| signer.verify(&received))
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

    /// RFC 4231, test case 2.
    const KEY: &[u8] = b"Jefe";
    const DATA: &[u8] = b"what do ya want for nothing?";

    fn sign(algorithm: &str, chunks: &[&[u8]]) -> String {
        let mut signer = Signer::new(algorithm, KEY).unwrap();
        for chunk in chunks {
            signer.update(chunk);
        }
        hex::encode(signer.sign())
    }

    #[test]
    fn signatures_match_rfc_4231() {
        assert_eq!(
            sign("sha256", &[DATA]),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        assert_eq!(
            sign("sha384", &[DATA]),
            "af45d2e376484031617f78d2b58a6b1b9c7ef464f5a01b47e42ec3736322445e8e2240ca5e69e2c78b3239ecfab21649"
        );
        assert_eq!(
            sign("sha512", &[DATA]),
            "164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea2505549758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737"
        );
    }

    #[test]
    fn data_can_come_in_chunks() {
        assert_eq!(
            sign("sha256", &[&DATA[..10], &DATA[10..]]),
            sign("sha256", &[DATA])
        );
    }

    #[test]
    fn signatures_are_verified() {
        let signature =
            hex::decode("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843")
                .unwrap();
        let mut signer = Signer::new("sha256", KEY).unwrap();
        signer.update(DATA);
        assert!(signer.verify(&signature));

        let mut signer = Signer::new("sha256", KEY).unwrap();
        signer.update(b"what do ya want for something?");
        assert!(!signer.verify(&signature));
    }

    #[test]
    fn signatures_are_encoded_and_decoded() {
        let bytes = [0x5b, 0xdc, 0xff];
        assert_eq!(encode(&bytes, "hex"), "5bdcff");
        assert_eq!(encode(&bytes, "base64"), "W9z/");
        assert_eq!(decode("5BDCFF", "hex"), Some(bytes.to_vec()));
        assert_eq!(decode("W9z/", "base64"), Some(bytes.to_vec()));
        assert_eq!(decode("not hex", "hex"), None);
    }

    #[test]
    fn unsupported_settings_are_refused() {
        assert!(Signer::new("md5", KEY).is_err());
        assert!(check_encoding("base32").is_err());
    }
}
