use crate::descriptor::{DataType, Identifier};
use crate::executive::{PackedArray, Secret, SecretId, TrackId, Value};
use async_trait::async_trait;
use core::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Weak};

/// Resolves the value of secrets designated by a locator scheme.
///
/// Sources are registered under their scheme, a `vault` source resolving `vault:<path>`
/// locators, by models declaring `secret_sources` in `#[mel_model]`, before any model is
/// initialized. `env` and `file` are built in.
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
}

/// Computes the value of a derived secret from the values of its input secrets.
///
/// Derivations are built by functions, they describe an operation and never hold
/// any secret value themselves.
pub trait SecretDerivation: Debug + Send + Sync {
    /// Gives the derived value from `inputs`, the values of the input secrets, in order.
    ///
    /// Errors must describe what failed without including any part of the values.
    fn derive(&self, inputs: &[Value]) -> Result<Value, String>;
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
    /// Records an access to a secret, as a debug event, and also in the log
    /// for denials and resolution failures.
    async fn secret_audit(&self, audit: SecretAudit);
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
                        _ => return Err(SecretError::MismatchingValue),
                    }
                }
                Ok(f(&bytes))
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

    /// Resolves the value of `secret` and drops it right away,
    /// to check that it can be resolved.
    ///
    /// The value is not given to anyone, so the access is not recorded.
    pub async fn check_resolution(&self, secret: &Secret) -> Result<(), SecretError> {
        let host = self.host.upgrade().ok_or(SecretError::WorldEnded)?;
        secret.check_resolution(&*host).await
    }

    /// Registers `source` for the `scheme` of locators, such as `vault` for `vault:<path>`.
    ///
    /// Only possible from the `secret_sources` function of a model, called before any
    /// model is initialized. Registering a scheme twice, or `env` and `file`, makes
    /// the launch fail.
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
