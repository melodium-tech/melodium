//! JSON Web Tokens (RFC 7519), signed with HMAC, RSA, ECDSA or Ed25519.
//!
//! `sign` gives tokens as secrets, as they are credentials, and `verify` checks received
//! tokens against a shared secret or a public key.

use crate::encoding::{base64url, from_base64url};
use crate::signature::hmac::Signer as HmacSigner;
use core::str::FromStr;
use melodium_core::common::{
    descriptor::DataType,
    executive::{
        Secret as ExecutiveSecret, SecretOrigin, SecretPolicy, SecretReveal, SecretTransmission,
    },
};
use melodium_core::*;
use melodium_macro::mel_treatment;
use p256::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use rsa::signature::{Signer, Verifier};
use serde_json::{Map as JsonMap, Value as Json};
use sha2::{Sha256, Sha384, Sha512};
use zeroize::Zeroizing;

/// Algorithms of the `alg` header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Algorithm {
    Hs256,
    Hs384,
    Hs512,
    Rs256,
    Rs384,
    Rs512,
    Es256,
    Es384,
    EdDsa,
}

const ALGORITHMS: &str =
    "'HS256', 'HS384', 'HS512', 'RS256', 'RS384', 'RS512', 'ES256', 'ES384' or 'EdDSA'";

impl FromStr for Algorithm {
    type Err = String;

    fn from_str(algorithm: &str) -> Result<Self, Self::Err> {
        match algorithm {
            "HS256" => Ok(Self::Hs256),
            "HS384" => Ok(Self::Hs384),
            "HS512" => Ok(Self::Hs512),
            "RS256" => Ok(Self::Rs256),
            "RS384" => Ok(Self::Rs384),
            "RS512" => Ok(Self::Rs512),
            "ES256" => Ok(Self::Es256),
            "ES384" => Ok(Self::Es384),
            "EdDSA" => Ok(Self::EdDsa),
            _ => Err(format!(
                "'{algorithm}' is not a supported algorithm, expected {ALGORITHMS}"
            )),
        }
    }
}

impl Algorithm {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Hs256 => "HS256",
            Self::Hs384 => "HS384",
            Self::Hs512 => "HS512",
            Self::Rs256 => "RS256",
            Self::Rs384 => "RS384",
            Self::Rs512 => "RS512",
            Self::Es256 => "ES256",
            Self::Es384 => "ES384",
            Self::EdDsa => "EdDSA",
        }
    }

    fn hmac(&self) -> Option<&'static str> {
        match self {
            Self::Hs256 => Some("sha256"),
            Self::Hs384 => Some("sha384"),
            Self::Hs512 => Some("sha512"),
            _ => None,
        }
    }
}

/// Seconds since the Unix epoch.
#[cfg(feature = "real")]
fn now() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .map_err(|_| "the clock is before 1970".to_string())
}

#[cfg(feature = "mock")]
fn now() -> Result<u64, String> {
    Err("the clock is not available in mock mode".to_string())
}

/// Signs `message` with `key`, the shared secret for HMAC, or a PEM private key.
///
/// Errors never include any part of the key.
pub(crate) fn sign_message(
    algorithm: Algorithm,
    key: &str,
    message: &[u8],
) -> Result<Vec<u8>, String> {
    let not_fitting = || format!("the key is not a PEM private key for {}", algorithm.name());
    match algorithm {
        Algorithm::Hs256 | Algorithm::Hs384 | Algorithm::Hs512 => {
            let mut signer = HmacSigner::new(algorithm.hmac().unwrap(), key.as_bytes())?;
            signer.update(message);
            Ok(signer.sign())
        }
        Algorithm::Rs256 | Algorithm::Rs384 | Algorithm::Rs512 => {
            let key = rsa::RsaPrivateKey::from_pkcs8_pem(key)
                .or_else(|_| rsa::RsaPrivateKey::from_pkcs1_pem(key))
                .map_err(|_| not_fitting())?;
            sign_rsa(algorithm, key, message)
        }
        Algorithm::Es256 => {
            let key = p256::ecdsa::SigningKey::from_pkcs8_pem(key)
                .or_else(|_| p256::SecretKey::from_sec1_pem(key).map(Into::into))
                .map_err(|_| not_fitting())?;
            let signature: p256::ecdsa::Signature = key.sign(message);
            Ok(signature.to_bytes().to_vec())
        }
        Algorithm::Es384 => {
            let key = p384::ecdsa::SigningKey::from_pkcs8_pem(key)
                .or_else(|_| p384::SecretKey::from_sec1_pem(key).map(Into::into))
                .map_err(|_| not_fitting())?;
            let signature: p384::ecdsa::Signature = key.sign(message);
            Ok(signature.to_bytes().to_vec())
        }
        Algorithm::EdDsa => {
            let key = ed25519_dalek::SigningKey::from_pkcs8_pem(key).map_err(|_| not_fitting())?;
            Ok(key.sign(message).to_bytes().to_vec())
        }
    }
}

/// Signs with RSA PKCS#1 v1.5, blinded with random numbers against timing attacks.
#[cfg(feature = "real")]
fn sign_rsa(
    algorithm: Algorithm,
    key: rsa::RsaPrivateKey,
    message: &[u8],
) -> Result<Vec<u8>, String> {
    use rsa::pkcs1v15::SigningKey;
    use rsa::signature::{RandomizedSigner, SignatureEncoding};
    let mut rng = rsa::rand_core::UnwrapErr(getrandom::SysRng);
    Ok(match algorithm {
        Algorithm::Rs256 => SigningKey::<Sha256>::new(key)
            .sign_with_rng(&mut rng, message)
            .to_vec(),
        Algorithm::Rs384 => SigningKey::<Sha384>::new(key)
            .sign_with_rng(&mut rng, message)
            .to_vec(),
        _ => SigningKey::<Sha512>::new(key)
            .sign_with_rng(&mut rng, message)
            .to_vec(),
    })
}

#[cfg(feature = "mock")]
fn sign_rsa(
    _algorithm: Algorithm,
    _key: rsa::RsaPrivateKey,
    _message: &[u8],
) -> Result<Vec<u8>, String> {
    Err("RSA signing is not available in mock mode".to_string())
}

/// Public key checking signatures for one of the algorithms not using HMAC.
#[derive(Debug)]
pub(crate) enum PublicKey {
    Rsa(rsa::RsaPublicKey),
    P256(p256::ecdsa::VerifyingKey),
    P384(p384::ecdsa::VerifyingKey),
    Ed25519(ed25519_dalek::VerifyingKey),
}

impl PublicKey {
    /// Reads `pem`, a public key for `algorithm`.
    pub(crate) fn from_pem(algorithm: Algorithm, pem: &str) -> Result<Self, String> {
        let not_fitting = || format!("the key is not a PEM public key for {}", algorithm.name());
        match algorithm {
            Algorithm::Hs256 | Algorithm::Hs384 | Algorithm::Hs512 => Err(format!(
                "{} tokens are signed with a shared secret, not a public key",
                algorithm.name()
            )),
            Algorithm::Rs256 | Algorithm::Rs384 | Algorithm::Rs512 => {
                rsa::RsaPublicKey::from_public_key_pem(pem)
                    .or_else(|_| rsa::RsaPublicKey::from_pkcs1_pem(pem))
                    .map(Self::Rsa)
                    .map_err(|_| not_fitting())
            }
            Algorithm::Es256 => p256::ecdsa::VerifyingKey::from_public_key_pem(pem)
                .map(Self::P256)
                .map_err(|_| not_fitting()),
            Algorithm::Es384 => p384::ecdsa::VerifyingKey::from_public_key_pem(pem)
                .map(Self::P384)
                .map_err(|_| not_fitting()),
            Algorithm::EdDsa => ed25519_dalek::VerifyingKey::from_public_key_pem(pem)
                .map(Self::Ed25519)
                .map_err(|_| not_fitting()),
        }
    }

    /// Tells whether `signature` is the signature of `message` with `algorithm`.
    pub(crate) fn verify(&self, algorithm: Algorithm, message: &[u8], signature: &[u8]) -> bool {
        match self {
            Self::Rsa(key) => {
                use rsa::pkcs1v15::{Signature, VerifyingKey};
                let Ok(signature) = Signature::try_from(signature) else {
                    return false;
                };
                match algorithm {
                    Algorithm::Rs256 => VerifyingKey::<Sha256>::new(key.clone())
                        .verify(message, &signature)
                        .is_ok(),
                    Algorithm::Rs384 => VerifyingKey::<Sha384>::new(key.clone())
                        .verify(message, &signature)
                        .is_ok(),
                    Algorithm::Rs512 => VerifyingKey::<Sha512>::new(key.clone())
                        .verify(message, &signature)
                        .is_ok(),
                    _ => false,
                }
            }
            Self::P256(key) => p256::ecdsa::Signature::from_slice(signature)
                .map(|signature| key.verify(message, &signature).is_ok())
                .unwrap_or(false),
            Self::P384(key) => p384::ecdsa::Signature::from_slice(signature)
                .map(|signature| key.verify(message, &signature).is_ok())
                .unwrap_or(false),
            Self::Ed25519(key) => crate::signature::ed25519::verified(key, message, signature),
        }
    }
}

/// Key checking the signature of a token: a public key, or the secret shared with the
/// issuer for HMAC.
pub(crate) enum Key<'a> {
    Public(&'a PublicKey),
    Shared(&'a [u8]),
}

impl Key<'_> {
    fn verify(&self, algorithm: Algorithm, message: &[u8], signature: &[u8]) -> bool {
        match (self, algorithm.hmac()) {
            (Self::Public(key), None) => key.verify(algorithm, message, signature),
            (Self::Shared(secret), Some(hash)) => HmacSigner::new(hash, secret)
                .map(|mut signer| {
                    signer.update(message);
                    signer.verify(signature)
                })
                .unwrap_or(false),
            _ => false,
        }
    }
}

/// Makes a token of `claims` signed with `key`, valid for `lifetime` seconds from `now`
/// when it is not `0`.
pub(crate) fn token(
    algorithm: Algorithm,
    key: &str,
    claims: &str,
    lifetime: u64,
    now: impl FnOnce() -> Result<u64, String>,
) -> Result<String, String> {
    let mut claims: JsonMap<String, Json> =
        serde_json::from_str(claims).map_err(|_| "the claims are not a JSON object".to_string())?;
    if lifetime > 0 {
        let now = now()?;
        // A minute in the past, for clocks running slightly ahead of the receiver's.
        claims.insert("iat".to_string(), Json::from(now.saturating_sub(60)));
        claims.insert("exp".to_string(), Json::from(now.saturating_add(lifetime)));
    }
    let header = serde_json::json!({ "alg": algorithm.name(), "typ": "JWT" });
    let mut token = format!(
        "{}.{}",
        base64url(header.to_string().as_bytes()),
        base64url(Json::Object(claims).to_string().as_bytes())
    );
    let signature = sign_message(algorithm, key, token.as_bytes())?;
    token.push('.');
    token.push_str(&base64url(&signature));
    Ok(token)
}

/// Checks `token` against `key` for `algorithm`, at `now`, giving its claims as JSON,
/// or why it is not valid.
pub(crate) fn check(
    token: &str,
    algorithm: Algorithm,
    key: &Key,
    issuer: &str,
    audience: &str,
    leeway: u64,
    now: u64,
) -> Result<String, String> {
    let parts: Vec<&str> = token.split('.').collect();
    let [header, payload, signature] = parts[..] else {
        return Err("the token is not made of three parts".to_string());
    };
    let signed = &token[..header.len() + 1 + payload.len()];
    let header: JsonMap<String, Json> = from_base64url(header)
        .and_then(|header| serde_json::from_slice(&header).ok())
        .ok_or_else(|| "the token header is not valid".to_string())?;
    match header.get("alg").and_then(Json::as_str) {
        Some(alg) if alg == algorithm.name() => {}
        Some(alg) => {
            return Err(format!(
                "the token is signed with '{alg}', not '{}'",
                algorithm.name()
            ))
        }
        None => return Err("the token header gives no algorithm".to_string()),
    }
    if header.contains_key("crit") {
        return Err("the token has critical header parameters".to_string());
    }
    let signature =
        from_base64url(signature).ok_or_else(|| "the token signature is not valid".to_string())?;
    if !key.verify(algorithm, signed.as_bytes(), &signature) {
        return Err("the token signature does not match".to_string());
    }

    let payload = from_base64url(payload)
        .and_then(|payload| String::from_utf8(payload).ok())
        .ok_or_else(|| "the token claims are not valid".to_string())?;
    let claims: JsonMap<String, Json> = serde_json::from_str(&payload)
        .map_err(|_| "the token claims are not a JSON object".to_string())?;
    let now = now as f64;
    let leeway = leeway as f64;
    if let Some(expiration) = claims.get("exp").and_then(Json::as_f64) {
        if now > expiration + leeway {
            return Err("the token has expired".to_string());
        }
    }
    if let Some(not_before) = claims.get("nbf").and_then(Json::as_f64) {
        if now + leeway < not_before {
            return Err("the token is not valid yet".to_string());
        }
    }
    if !issuer.is_empty() && claims.get("iss").and_then(Json::as_str) != Some(issuer) {
        return Err("the token is not from the expected issuer".to_string());
    }
    if !audience.is_empty() {
        let intended = match claims.get("aud") {
            Some(Json::String(aud)) => aud == audience,
            Some(Json::Array(auds)) => auds.iter().any(|aud| aud.as_str() == Some(audience)),
            _ => false,
        };
        if !intended {
            return Err("the token is not for the expected audience".to_string());
        }
    }
    Ok(payload)
}

/// Signs a token of `claims` with `key`, giving it as a secret named `name`.
async fn signed_token(
    access: &melodium_core::common::executive::SecretsAccess,
    key: &ExecutiveSecret,
    algorithm: &str,
    claims: &str,
    lifetime: u64,
    name: String,
    transmission: &str,
    plain_reveal: bool,
) -> Result<ExecutiveSecret, String> {
    let algorithm = Algorithm::from_str(algorithm)?;
    let transmission = transmission
        .parse::<SecretTransmission>()
        .map_err(|error| error.to_string())?;
    let token = access
        .reveal_str(key, |key| token(algorithm, key, claims, lifetime, now))
        .await
        .map_err(|error| format!("key: {error}"))??;
    ExecutiveSecret::new(
        name,
        DataType::String,
        SecretPolicy {
            transmission,
            reveal: SecretReveal::Any,
            plain_reveal,
        },
        SecretOrigin::Inline(Value::String(token)),
    )
    .map_err(|error| error.to_string())
}

/// Signs a JSON Web Token, given as a secret.
///
/// Once `claims` is received, a JSON object such as `{"iss": "123456"}`, the token is signed
/// with `key` and `algorithm`, and given through `token`:
/// - `key`: for `HS256`, `HS384` and `HS512`, the secret shared with the receiver; for
///   `RS256`, `RS384` and `RS512` (RSA), `ES256` and `ES384` (ECDSA with P-256 and P-384),
///   and `EdDSA` (Ed25519), the private key in PEM, PKCS#8 (`-----BEGIN PRIVATE KEY-----`)
///   or the `RSA PRIVATE KEY` and `EC PRIVATE KEY` forms, such as GitHub App keys.
/// - `algorithm`: one of the above, written in the token header.
/// - `lifetime`: when not `0`, the token is valid for `lifetime` seconds from its signing:
///   `exp` is set accordingly, and `iat` a minute before signing, for clocks running
///   slightly ahead of the receiver's.
///
/// The key is revealed only to sign. The token is a secret named `name`, kept in memory
/// until it is dropped, whose policy is given by `transmission` (`local`, `reference` or
/// `value`) and `plain_reveal` (whether the plain `std/secret::reveal` treatment may reveal
/// it), as `std/secret::conceal` takes. Programs running longer than `lifetime` sign again
/// before the token expires, as receivers refuse expired tokens.
///
/// Signing takes microseconds with HMAC and Ed25519, half a millisecond with ECDSA, and a
/// few milliseconds with RSA, more with keys longer than 2048 bits.
///
/// If the key cannot be revealed or does not fit `algorithm`, `algorithm` is not supported,
/// `claims` is not a JSON object, or `transmission` is not valid, `failed` is emitted and
/// `error` contains the reason.
#[mel_treatment(
    input claims Block<string>
    output token Block<Secret<string>>
    output failed Block<void>
    output error Block<string>
    default lifetime 0
    default transmission "local"
    default plain_reveal false
    secrets_access
)]
pub async fn sign(
    key: Secret<string>,
    algorithm: string,
    lifetime: u64,
    name: string,
    transmission: string,
    plain_reveal: bool,
) {
    if let Ok(received) = claims.recv_one_as::<String>().await {
        match signed_token(
            &secrets_access,
            &key,
            &algorithm,
            &received,
            lifetime,
            name,
            &transmission,
            plain_reveal,
        )
        .await
        {
            Ok(signed) => {
                let _ = token.send_one(Value::Secret(signed)).await;
            }
            Err(message) => {
                let _ = failed.send_one_as(()).await;
                let _ = error.send_one_as(message).await;
            }
        }
    }
}

/// Key given to `verify`, according to the algorithm.
enum VerifyingKey {
    Public(PublicKey),
    Shared(ExecutiveSecret),
}

/// Verifies a received JSON Web Token.
///
/// `token`, after `prefix` when given (such as `Bearer ` for the `authorization` secret of
/// `@HttpRequest`), is checked for `algorithm`, which the token header must name:
/// - `HS256`, `HS384` and `HS512` against `secret`, the key shared with the issuer;
/// - `RS256`, `RS384`, `RS512`, `ES256`, `ES384` and `EdDSA` against `public_key`, a PEM
///   public key (`-----BEGIN PUBLIC KEY-----`).
///
/// A token naming another algorithm is invalid, so a public key is never taken as a shared
/// secret. Its `exp` and `nbf` claims, when given, must hold with a tolerance of `leeway`
/// seconds, and its `iss` and `aud` claims must be `issuer` and include `audience`, when
/// those are not empty. The token and `secret` are revealed only to be checked.
///
/// If the token is valid, `claims` gives its claims as JSON, then `valid` is emitted.
/// Otherwise `reason` tells why, then `invalid` is emitted, including when `token` is none
/// or is not a `Secret<string>`.
/// If `algorithm` is not valid, the key it needs is not given or not valid, or a secret
/// cannot be revealed, `failed` is emitted and `error` contains the reason.
#[mel_treatment(
    input token Block<Option<Secret<string>>>
    output claims Block<string>
    output valid Block<void>
    output reason Block<string>
    output invalid Block<void>
    output failed Block<void>
    output error Block<string>
    default public_key ""
    default secret none
    default prefix ""
    default issuer ""
    default audience ""
    default leeway 60
    secrets_access
)]
pub async fn verify(
    algorithm: string,
    public_key: string,
    secret: Option<Secret<string>>,
    prefix: string,
    issuer: string,
    audience: string,
    leeway: u64,
) {
    let prepared = Algorithm::from_str(&algorithm).and_then(|algorithm| {
        let key = match (algorithm.hmac(), secret) {
            (Some(_), Some(secret)) => VerifyingKey::Shared(secret),
            (Some(_), None) => {
                return Err(format!(
                    "{} tokens are checked with a secret, and none is given",
                    algorithm.name()
                ))
            }
            (None, _) => VerifyingKey::Public(PublicKey::from_pem(algorithm, &public_key)?),
        };
        Ok((algorithm, key, now()?))
    });
    let (algorithm, key, checked_at) = match prepared {
        Ok(prepared) => prepared,
        Err(message) => {
            let _ = failed.send_one_as(()).await;
            let _ = error.send_one_as(message).await;
            return;
        }
    };

    let received = match token.recv_one().await {
        Ok(Value::Option(Some(value))) => match *value {
            Value::Secret(received) => Some(received),
            _ => None,
        },
        _ => None,
    };
    let Some(received) = received else {
        let _ = reason.send_one_as("no token received".to_string()).await;
        let _ = invalid.send_one_as(()).await;
        return;
    };

    let check = |token: &str, key: &Key| match token.strip_prefix(prefix.as_str()) {
        Some(token) => check(
            token, algorithm, key, &issuer, &audience, leeway, checked_at,
        ),
        None => Err("the token does not start with the expected prefix".to_string()),
    };
    let checked = match &key {
        VerifyingKey::Public(public) => secrets_access
            .reveal_str(&received, |token| check(token, &Key::Public(public)))
            .await
            .map_err(|error| format!("token: {error}")),
        VerifyingKey::Shared(secret) => {
            match secrets_access
                .reveal_str(&received, |token| Zeroizing::new(token.to_string()))
                .await
            {
                Ok(token) => secrets_access
                    .reveal_str(secret, |shared| {
                        check(&token, &Key::Shared(shared.as_bytes()))
                    })
                    .await
                    .map_err(|error| format!("secret: {error}")),
                Err(error) => Err(format!("token: {error}")),
            }
        }
    };
    match checked {
        Ok(Ok(checked_claims)) => {
            let _ = claims.send_one_as(checked_claims).await;
            let _ = valid.send_one_as(()).await;
        }
        Ok(Err(why)) => {
            let _ = reason.send_one_as(why).await;
            let _ = invalid.send_one_as(()).await;
        }
        Err(message) => {
            let _ = failed.send_one_as(()).await;
            let _ = error.send_one_as(message).await;
        }
    }
}

#[cfg(all(test, feature = "real"))]
mod tests {
    use super::*;
    use p256::elliptic_curve::Generate;
    use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
    use rsa::pkcs1::EncodeRsaPrivateKey;

    const CLAIMS: &str = r#"{"iss":"123456","aud":"melodium"}"#;
    const NOW: u64 = 1_800_000_000;

    fn claims(token: &str) -> JsonMap<String, Json> {
        let payload = token.split('.').nth(1).unwrap();
        serde_json::from_slice(&from_base64url(payload).unwrap()).unwrap()
    }

    fn round_trip(algorithm: Algorithm, private: &str, public: &str) {
        let token = token(algorithm, private, CLAIMS, 600, || Ok(NOW)).unwrap();
        let key = PublicKey::from_pem(algorithm, public).unwrap();
        let checked = check(
            &token,
            algorithm,
            &Key::Public(&key),
            "123456",
            "melodium",
            60,
            NOW + 10,
        );
        assert!(checked.is_ok(), "{algorithm:?}: {checked:?}");

        let mut forged = token.clone();
        forged.insert_str(token.find('.').unwrap() + 1, "e30");
        assert!(check(&forged, algorithm, &Key::Public(&key), "", "", 60, NOW).is_err());
    }

    #[test]
    fn tokens_are_signed_and_checked_with_every_public_key_algorithm() {
        let mut rng = rsa::rand_core::UnwrapErr(getrandom::SysRng);
        let rsa = rsa::RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let rsa_public = rsa::RsaPublicKey::from(&rsa)
            .to_public_key_pem(LineEnding::LF)
            .unwrap();
        // GitHub App keys use the older PKCS#1 form.
        let rsa_pkcs1 = rsa.to_pkcs1_pem(LineEnding::LF).unwrap();
        let rsa_pkcs8 = rsa.to_pkcs8_pem(LineEnding::LF).unwrap();
        for algorithm in [Algorithm::Rs256, Algorithm::Rs384, Algorithm::Rs512] {
            round_trip(algorithm, &rsa_pkcs1, &rsa_public);
            round_trip(algorithm, &rsa_pkcs8, &rsa_public);
        }

        let p256 = p256::ecdsa::SigningKey::generate_from_rng(&mut rng);
        round_trip(
            Algorithm::Es256,
            &p256.to_pkcs8_pem(LineEnding::LF).unwrap(),
            &p256
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        );
        let p384 = p384::ecdsa::SigningKey::generate_from_rng(&mut rng);
        round_trip(
            Algorithm::Es384,
            &p384.to_pkcs8_pem(LineEnding::LF).unwrap(),
            &p384
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        );
        let ed25519 = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
        round_trip(
            Algorithm::EdDsa,
            &ed25519.to_pkcs8_pem(LineEnding::LF).unwrap(),
            &ed25519
                .verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        );
    }

    #[test]
    fn hmac_tokens_match_their_signature() {
        let token = token(Algorithm::Hs256, "shared", CLAIMS, 0, || Ok(NOW)).unwrap();
        let (signed, signature) = token.rsplit_once('.').unwrap();
        let mut signer = HmacSigner::new("sha256", b"shared").unwrap();
        signer.update(signed.as_bytes());
        assert!(signer.verify(&from_base64url(signature).unwrap()));
        let header: JsonMap<String, Json> =
            serde_json::from_slice(&from_base64url(token.split('.').next().unwrap()).unwrap())
                .unwrap();
        assert_eq!(header["alg"], "HS256");
        assert_eq!(header["typ"], "JWT");
        // Without lifetime, the claims are kept as given.
        assert_eq!(
            claims(&token),
            serde_json::from_str::<JsonMap<String, Json>>(CLAIMS).unwrap()
        );
    }

    #[test]
    fn hmac_tokens_are_checked_with_the_shared_secret() {
        let token = token(Algorithm::Hs256, "shared", CLAIMS, 600, || Ok(NOW)).unwrap();
        let check = |token: &str, algorithm: Algorithm, secret: &[u8]| {
            super::check(token, algorithm, &Key::Shared(secret), "", "", 60, NOW)
        };
        assert!(check(&token, Algorithm::Hs256, b"shared").is_ok());
        assert_eq!(
            check(&token, Algorithm::Hs256, b"other"),
            Err("the token signature does not match".to_string())
        );
        assert_eq!(
            check(&token, Algorithm::Hs384, b"shared"),
            Err("the token is signed with 'HS256', not 'HS384'".to_string())
        );

        // A public key given as shared secret never checks a token naming a public key
        // algorithm, nor the other way round.
        let ed25519 = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
        let signed = super::token(
            Algorithm::EdDsa,
            &ed25519.to_pkcs8_pem(LineEnding::LF).unwrap(),
            CLAIMS,
            600,
            || Ok(NOW),
        )
        .unwrap();
        let public = ed25519
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .unwrap();
        assert!(super::check(
            &signed,
            Algorithm::EdDsa,
            &Key::Shared(public.as_bytes()),
            "",
            "",
            60,
            NOW
        )
        .is_err());
        let key = PublicKey::from_pem(Algorithm::EdDsa, &public).unwrap();
        assert!(super::check(
            &token,
            Algorithm::Hs256,
            &Key::Public(&key),
            "",
            "",
            60,
            NOW
        )
        .is_err());
    }

    #[test]
    fn lifetimes_set_issue_and_expiration_times() {
        let token = token(Algorithm::Hs256, "shared", CLAIMS, 600, || Ok(NOW)).unwrap();
        let claims = claims(&token);
        assert_eq!(claims["iat"], NOW - 60);
        assert_eq!(claims["exp"], NOW + 600);
        assert_eq!(claims["iss"], "123456");
    }

    #[test]
    fn invalid_tokens_tell_why() {
        let key = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
        let private = key.to_pkcs8_pem(LineEnding::LF).unwrap();
        let public = PublicKey::from_pem(
            Algorithm::EdDsa,
            &key.verifying_key()
                .to_public_key_pem(LineEnding::LF)
                .unwrap(),
        )
        .unwrap();
        let token = token(Algorithm::EdDsa, &private, CLAIMS, 600, || Ok(NOW)).unwrap();
        let check = |token: &str, issuer: &str, audience: &str, now: u64| {
            super::check(
                token,
                Algorithm::EdDsa,
                &Key::Public(&public),
                issuer,
                audience,
                60,
                now,
            )
        };

        assert_eq!(
            check(&token, "", "", NOW + 661),
            Err("the token has expired".to_string())
        );
        assert_eq!(
            check(&token, "654321", "", NOW),
            Err("the token is not from the expected issuer".to_string())
        );
        assert_eq!(
            check(&token, "", "other", NOW),
            Err("the token is not for the expected audience".to_string())
        );
        assert_eq!(
            check("a.b", "", "", NOW),
            Err("the token is not made of three parts".to_string())
        );
        let hmac = super::token(Algorithm::Hs256, "shared", CLAIMS, 0, || Ok(NOW)).unwrap();
        assert_eq!(
            check(&hmac, "", "", NOW),
            Err("the token is signed with 'HS256', not 'EdDSA'".to_string())
        );
        let none = format!(
            "{}.{}.",
            base64url(br#"{"alg":"none"}"#),
            token.split('.').nth(1).unwrap()
        );
        assert!(check(&none, "", "", NOW).is_err());
    }

    #[test]
    fn keys_not_fitting_are_refused_without_showing_them() {
        let error = sign_message(Algorithm::Rs256, "not-a-key-sentinel", b"data").unwrap_err();
        assert!(!error.contains("sentinel"), "{error}");
        assert!(PublicKey::from_pem(Algorithm::Hs256, "").is_err());
        assert!("none".parse::<Algorithm>().is_err());
    }
}
