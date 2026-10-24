use crate::descriptor::{DataType, Identifier};
use crate::executive::{
    PackedArray, Secret, SecretId, SecretTransfer, SecretTransmission, TrackId, Value,
};
use async_trait::async_trait;
use core::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Weak};
use zeroize::Zeroize;

/// Resolves the value of secrets designated by a locator scheme.
///
/// Sources are registered under their scheme, a `vault` source resolving `vault:<path>`
/// locators, by models declaring `secret_sources` in `#[mel_model]`, before any model is
/// initialized. `env` and `file` are built in, and the program running the engine can give
/// others before genesis, such as `api` on Cadence.CI workers.
#[async_trait]
pub trait SecretSource: Debug + Send + Sync {
    /// Gives the value at `path` (the locator part following `<scheme>:`), as `datatype`.
    ///
    /// Errors must describe what failed without including any part of the value.
    async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String>;

    /// Whether the plain `std/secret::reveal` treatment may reveal secrets from this
    /// source, when their policy allows it.
    fn plain_reveal(&self) -> bool {
        true
    }

    /// Called when the engine ends, to wipe what the source keeps, such as cached values.
    ///
    /// Tracks still finishing may resolve secrets afterwards.
    fn shutdown(&self) {}
}

/// Computes the value of a derived secret from the values of its input secrets.
///
/// Derivations are built by functions, they describe an operation and never hold
/// any secret value themselves. They run each time the derived secret is revealed, within
/// the element revealing it, so they stay cheap, such as formatting or encoding: costly
/// operations, such as signing, belong to treatments.
pub trait SecretDerivation: Debug + Send + Sync {
    /// Gives the derived value from `inputs`, the values of the input secrets, in order.
    ///
    /// Errors must describe what failed without including any part of the values.
    fn derive(&self, inputs: &[Value]) -> Result<Value, String>;
}

/// Writes and reads secrets held inside data values crossing to a distant engine.
///
/// The distribution layer sets one with `with_secret_wire` around the serialization of a
/// data value, and secrets met there are written and read through it. Out of it, secrets
/// refuse to be serialized or deserialized.
pub trait SecretWire {
    /// Gives what to write for `secret`.
    ///
    /// Errors must describe what failed without including any part of the value.
    fn write(&self, secret: &Secret) -> Result<Box<dyn erased_serde::Serialize + '_>, String>;

    /// Reads a secret written by `write` on the sending engine.
    fn read<'de>(
        &self,
        deserializer: &mut dyn erased_serde::Deserializer<'de>,
    ) -> Result<Secret, String>;
}

scoped_tls::scoped_thread_local!(static SECRET_WIRE: Box<dyn SecretWire>);

/// Runs `f` with `wire` writing and reading the secrets serialized or deserialized meanwhile
/// on this thread.
pub fn with_secret_wire<R>(wire: &Box<dyn SecretWire>, f: impl FnOnce() -> R) -> R {
    SECRET_WIRE.set(wire, f)
}

/// Runs `f` with the secret wire set by `with_secret_wire`, if any.
pub(crate) fn current_secret_wire<R>(f: impl FnOnce(Option<&dyn SecretWire>) -> R) -> R {
    if SECRET_WIRE.is_set() {
        SECRET_WIRE.with(|wire| f(Some(wire.as_ref())))
    } else {
        f(None)
    }
}

/// Engine side of secrets: registered sources, audit, and the phases of a run.
///
/// Implemented by the engine only, and never given to elements: they reach secrets
/// through the [`SecretsAccess`] the engine gives them.
#[doc(hidden)]
#[async_trait]
pub trait SecretsHost: Debug + Send + Sync {
    /// Gives the secret source registered for `scheme`.
    fn secret_source(&self, scheme: &str) -> Option<Arc<dyn SecretSource>>;
    /// Registers a secret source for `scheme`.
    ///
    /// Only possible while the engine registers the sources of its models, before
    /// any model is initialized, and only once for each scheme.
    fn register_secret_source(
        &self,
        scheme: &str,
        source: Arc<dyn SecretSource>,
    ) -> Result<(), SecretError>;
    /// Whether elements may reveal secrets, which they can once every model is initialized.
    fn revealing(&self) -> bool;
    /// Registers a value revealed from the secret named `secret_name`,
    /// masked from then on in log messages and remote debug text.
    fn add_masked_value(&self, secret_name: &str, value: &Value);
    /// Gives the values revealed so far, and from then on, to search them in data.
    fn revealed_values(&self) -> Option<Arc<dyn RevealedValues>> {
        None
    }
    /// Records an access to a secret, as a debug event, and also in the log
    /// for denials and resolution failures.
    async fn secret_audit(&self, audit: SecretAudit);
}

/// Values revealed by an engine, searched in data leaving it, such as files a program uploads.
///
/// Each value is searched as is, in base64 and URL-encoded, never line by line as masking
/// does, so that ordinary lines of a multi-line value are not found everywhere.
#[doc(hidden)]
pub trait RevealedValues: Send + Sync {
    /// Gives the name of the secret whose revealed value is found in `data`, if any.
    fn find(&self, data: &[u8]) -> Option<String>;
    /// Length of the longest value searched, in bytes.
    fn longest(&self) -> usize;
}

/// Search of the values revealed by the engine in data given chunk by chunk,
/// finding values across chunks too.
///
/// Values revealed while the search goes on are searched from then on.
pub struct RevealedScan {
    values: Option<Arc<dyn RevealedValues>>,
    /// End of the data searched so far, as long as the longest value but one byte.
    tail: Vec<u8>,
}

impl RevealedScan {
    /// Search finding nothing, for engines revealing nothing.
    pub fn none() -> Self {
        Self {
            values: None,
            tail: Vec::new(),
        }
    }

    /// Searches `chunk`, following the chunks searched before, and gives the name of the
    /// secret whose revealed value is found, if any.
    pub fn scan(&mut self, chunk: &[u8]) -> Option<String> {
        let values = self.values.as_ref()?;
        let kept = values.longest().saturating_sub(1);
        if kept == 0 {
            return None;
        }

        let found = values.find(chunk).or_else(|| {
            // Values across the end of the previous data and the start of this chunk.
            let mut boundary = self.tail.clone();
            boundary.extend_from_slice(&chunk[..chunk.len().min(kept)]);
            let found = values.find(&boundary);
            boundary.zeroize();
            found
        });

        if chunk.len() >= kept {
            self.tail.zeroize();
            self.tail.clear();
            self.tail.extend_from_slice(&chunk[chunk.len() - kept..]);
        } else {
            self.tail.extend_from_slice(chunk);
            let excess = self.tail.len().saturating_sub(kept);
            self.tail[..excess].zeroize();
            self.tail.drain(..excess);
        }
        found
    }
}

impl Drop for RevealedScan {
    fn drop(&mut self) {
        self.tail.zeroize();
    }
}

impl Debug for RevealedScan {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RevealedScan").finish_non_exhaustive()
    }
}

/// Resolves the value of `secret` and drops it right away, for the engine itself,
/// to check that it can be resolved.
#[doc(hidden)]
pub async fn check_secret_resolution(
    host: &dyn SecretsHost,
    secret: &Secret,
) -> Result<(), SecretError> {
    secret.check_resolution(host).await
}

/// Access to secrets, given by the engine to the elements declaring `secrets_access`.
///
/// Treatments get it as `secrets_access`, and models through `secrets_access()`.
/// It carries the identity of the element instance, set by the engine: every reveal
/// is checked against the policy of the secret for that element, and recorded.
#[derive(Clone)]
pub struct SecretsAccess {
    pub(crate) host: Weak<dyn SecretsHost>,
    pub(crate) element: Identifier,
    pub(crate) label: Option<String>,
    pub(crate) track_id: Option<TrackId>,
}

impl SecretsAccess {
    /// Made by the engine for the elements it builds.
    #[doc(hidden)]
    pub fn new(
        host: &Arc<dyn SecretsHost>,
        element: Identifier,
        label: Option<String>,
        track_id: Option<TrackId>,
    ) -> Self {
        Self {
            host: Arc::downgrade(host),
            element,
            label,
            track_id,
        }
    }

    /// Lends the value of `secret` to `f`.
    ///
    /// The policy is checked, the value resolved, and the access recorded:
    /// denials and resolution failures are also logged as errors.
    /// The value is dropped once `f` returns.
    pub async fn reveal<R>(
        &self,
        secret: &Secret,
        f: impl FnOnce(&Value) -> R + Send,
    ) -> Result<R, SecretError> {
        secret.access(self, false, f).await
    }

    /// Same as `reveal`, for a `Secret<string>`.
    pub async fn reveal_str<R>(
        &self,
        secret: &Secret,
        f: impl FnOnce(&str) -> R + Send,
    ) -> Result<R, SecretError> {
        self.reveal(secret, |value| match value {
            Value::String(value) => Ok(f(value)),
            _ => Err(SecretError::MismatchingValue),
        })
        .await?
    }

    /// Same as `reveal`, for a `Secret<Vec<byte>>`.
    pub async fn reveal_bytes<R>(
        &self,
        secret: &Secret,
        f: impl FnOnce(&[u8]) -> R + Send,
    ) -> Result<R, SecretError> {
        self.reveal(secret, |value| match value {
            Value::Packed(PackedArray::Byte(bytes)) | Value::Packed(PackedArray::U8(bytes)) => {
                Ok(f(bytes))
            }
            Value::Vec(values) => {
                let mut bytes = Vec::with_capacity(values.len());
                for value in values {
                    match value {
                        Value::Byte(byte) | Value::U8(byte) => bytes.push(*byte),
                        _ => {
                            bytes.zeroize();
                            return Err(SecretError::MismatchingValue);
                        }
                    }
                }
                let result = f(&bytes);
                bytes.zeroize();
                Ok(result)
            }
            _ => Err(SecretError::MismatchingValue),
        })
        .await?
    }

    /// Lends the value of `secret` to `f`, for the plain `std/secret::reveal` treatment.
    ///
    /// Same as `reveal`, but also requires the policy and the source of the secret
    /// to allow plain reveal.
    pub async fn reveal_plainly<R>(
        &self,
        secret: &Secret,
        f: impl FnOnce(&Value) -> R + Send,
    ) -> Result<R, SecretError> {
        secret.access(self, true, f).await
    }

    /// Gives what crosses to a distant engine for `secret`, following its transmission
    /// policy:
    /// - `local` secrets are refused;
    /// - `reference` secrets give their locator, those without locator are refused;
    /// - `value` secrets give their resolved value, over an encrypted connection only.
    ///
    /// Transmissions and refusals are recorded, refusals also in the log.
    pub async fn transmit(
        &self,
        secret: &Secret,
        encrypted: bool,
    ) -> Result<SecretTransfer, SecretError> {
        secret.transmit(self, encrypted).await
    }

    /// Resolves the value of `secret` and drops it right away,
    /// to check that it can be resolved.
    ///
    /// The value is not given to anyone, so the access is not recorded.
    pub async fn check_resolution(&self, secret: &Secret) -> Result<(), SecretError> {
        let host = self.host.upgrade().ok_or(SecretError::WorldEnded)?;
        secret.check_resolution(&*host).await
    }

    /// Masks `value` in log messages and remote debug texts from now on, as revealed values
    /// are, for values a source obtains by itself, such as the access tokens it logs in with.
    pub fn add_masked_value(&self, secret_name: &str, value: &Value) {
        if let Some(host) = self.host.upgrade() {
            host.add_masked_value(secret_name, value);
        }
    }

    /// Starts a search of the values the engine revealed in data given chunk by chunk,
    /// such as files leaving the engine, so that elements refuse to send revealed values
    /// where they would no longer be masked.
    ///
    /// The search only knows the values revealed by this engine, not by distant ones.
    pub fn revealed_scan(&self) -> RevealedScan {
        RevealedScan {
            values: self.host.upgrade().and_then(|host| host.revealed_values()),
            tail: Vec::new(),
        }
    }

    /// Registers `source` for the `scheme` of locators, such as `vault` for `vault:<path>`.
    ///
    /// Only possible from the `secret_sources` function of a model, called before any
    /// model is initialized. Registering a scheme twice, or one the engine already has,
    /// such as `env`, `file`, `api` and `api-id`, makes the launch fail.
    pub fn register_source(
        &self,
        scheme: &str,
        source: Arc<dyn SecretSource>,
    ) -> Result<(), SecretError> {
        let host = self.host.upgrade().ok_or(SecretError::WorldEnded)?;
        host.register_secret_source(scheme, source)
    }
}

impl Debug for SecretsAccess {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SecretsAccess")
            .field("element", &self.element)
            .field("label", &self.label)
            .field("track_id", &self.track_id)
            .finish_non_exhaustive()
    }
}

/// Outcome of an access to a secret.
#[derive(Clone, Debug, PartialEq)]
pub enum SecretAuditOutcome {
    Revealed,
    /// Sent to a distant engine, by reference or by value.
    Transmitted(SecretTransmission),
    Denied(String),
    ResolveFailed(String),
}

/// Record of an access to a secret, given to the world for audit.
#[derive(Clone, Debug)]
pub struct SecretAudit {
    pub secret_id: SecretId,
    pub secret_name: String,
    pub element: Identifier,
    pub label: Option<String>,
    pub track_id: Option<TrackId>,
    pub outcome: SecretAuditOutcome,
}

impl Display for SecretAudit {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let accessor = match &self.label {
            Some(label) => format!("{} (as '{label}')", self.element),
            None => self.element.to_string(),
        };
        match &self.outcome {
            SecretAuditOutcome::Revealed => {
                write!(f, "secret {:?} revealed to {accessor}", self.secret_name)
            }
            SecretAuditOutcome::Transmitted(transmission) => write!(
                f,
                "secret {:?} sent by {transmission} to a distant engine by {accessor}",
                self.secret_name
            ),
            SecretAuditOutcome::Denied(reason) => write!(
                f,
                "secret {:?} denied to {accessor}: {reason}",
                self.secret_name
            ),
            SecretAuditOutcome::ResolveFailed(error) => write!(
                f,
                "secret {:?} could not be resolved for {accessor}: {error}",
                self.secret_name
            ),
        }
    }
}

/// Error about a secret. Never includes any part of a secret value.
#[derive(Clone, Debug, PartialEq)]
pub enum SecretError {
    /// The type contains a secret, and a secret cannot hold another secret.
    NestedSecret(DataType),
    /// The inline value is not of the secret type.
    MismatchingValue,
    /// The locator is not `<scheme>:<path>`.
    InvalidLocator(String),
    /// The transmission policy is not `local`, `reference` or `value`.
    InvalidTransmission(String),
    /// The policy does not allow the access.
    Denied(String),
    /// The value could not be resolved.
    ResolveFailed(String),
    /// The world the access belongs to has ended.
    WorldEnded,
    /// Secret sources can only be registered before any model is initialized.
    RegistrationClosed,
    /// A source is already registered for the scheme.
    SourceAlreadyRegistered(String),
}

impl Display for SecretError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SecretError::NestedSecret(datatype) => {
                write!(f, "a secret cannot hold {datatype}, that contains a secret")
            }
            SecretError::MismatchingValue => write!(f, "the value is not of the secret type"),
            SecretError::InvalidLocator(locator) => write!(
                f,
                "'{locator}' is not a secret locator, expected '<scheme>:<path>' such as 'env:MELODIUM_SECRET_NAME' or 'file:PATH'"
            ),
            SecretError::InvalidTransmission(transmission) => write!(
                f,
                "'{transmission}' is not a secret transmission, expected 'local', 'reference' or 'value'"
            ),
            SecretError::Denied(reason) => write!(f, "access denied: {reason}"),
            SecretError::ResolveFailed(error) => write!(f, "resolution failed: {error}"),
            SecretError::WorldEnded => write!(f, "the engine has ended"),
            SecretError::RegistrationClosed => write!(
                f,
                "secret sources can only be registered before any model is initialized"
            ),
            SecretError::SourceAlreadyRegistered(scheme) => {
                write!(f, "a secret source is already registered for '{scheme}:'")
            }
        }
    }
}

impl std::error::Error for SecretError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Values searched with a plain search, `name` being found for each one.
    struct Values(Vec<(&'static [u8], &'static str)>);

    impl RevealedValues for Values {
        fn find(&self, data: &[u8]) -> Option<String> {
            self.0
                .iter()
                .find(|(value, _)| data.windows(value.len()).any(|window| window == *value))
                .map(|(_, name)| name.to_string())
        }

        fn longest(&self) -> usize {
            self.0
                .iter()
                .map(|(value, _)| value.len())
                .max()
                .unwrap_or(0)
        }
    }

    fn scan(values: Vec<(&'static [u8], &'static str)>) -> RevealedScan {
        RevealedScan {
            values: Some(Arc::new(Values(values))),
            tail: Vec::new(),
        }
    }

    #[test]
    fn finds_values_within_and_across_chunks() {
        let mut within = scan(vec![(b"revealed-value", "token")]);
        assert_eq!(
            within.scan(b"some revealed-value here"),
            Some("token".to_string())
        );

        let mut across = scan(vec![(b"revealed-value", "token")]);
        assert_eq!(across.scan(b"some reve"), None);
        assert_eq!(across.scan(b"aled-va"), None);
        assert_eq!(across.scan(b"lue and more"), Some("token".to_string()));
    }

    #[test]
    fn finds_nothing_in_other_data() {
        let mut other = scan(vec![(b"revealed-value", "token")]);
        for chunk in [&b"revealed"[..], b"-", b"other", b"revealed-valu"] {
            assert_eq!(other.scan(chunk), None);
        }

        let mut nothing_revealed = RevealedScan::none();
        assert_eq!(nothing_revealed.scan(b"revealed-value"), None);
    }
}
