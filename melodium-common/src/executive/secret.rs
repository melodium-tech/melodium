use crate::descriptor::{DataType, Identifier};
use crate::executive::{SecretId, TrackId, Value, World};
use async_trait::async_trait;
use core::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Weak};

/// Resolves the value of secrets designated by a locator scheme.
///
/// Sources are registered on the world under their scheme: a `vault` source
/// resolves `vault:<path>` locators. `env` and `file` are built in.
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

/// Identity of an element revealing secrets.
///
/// Rust elements get one from `#[mel_treatment]` (as `secret_access`)
/// and `#[mel_model]` (through `secret_access()`).
#[derive(Clone)]
pub struct SecretAccess {
    world: Weak<dyn World>,
    element: Identifier,
    label: Option<String>,
    track_id: Option<TrackId>,
}

impl SecretAccess {
    pub fn new(
        world: &Arc<dyn World>,
        element: Identifier,
        label: Option<String>,
        track_id: Option<TrackId>,
    ) -> Self {
        Self {
            world: Arc::downgrade(world),
            element,
            label,
            track_id,
        }
    }

    pub fn world(&self) -> Option<Arc<dyn World>> {
        self.world.upgrade()
    }

    pub fn element(&self) -> &Identifier {
        &self.element
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn track_id(&self) -> Option<TrackId> {
        self.track_id
    }
}

impl Debug for SecretAccess {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SecretAccess")
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
        }
    }
}

impl std::error::Error for SecretError {}
