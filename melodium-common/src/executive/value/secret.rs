use super::Value;
use crate::descriptor::DataType;
use crate::executive::secret::current_secret_wire;
use crate::executive::{
    count_wiped, register_wipe, wipe_value, SecretAudit, SecretAuditOutcome, SecretDerivation,
    SecretError, SecretsAccess, SecretsHost, Wipe,
};
use core::fmt::{Debug, Display, Formatter, Result};
use core::future::Future;
use core::pin::Pin;
use core::str::FromStr;
use core::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

pub type SecretId = u64;

static NEXT_SECRET_ID: AtomicU64 = AtomicU64::new(1);

/// Where a secret may go when a program is distributed.
///
/// Ordered from the most to the least restrictive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecretTransmission {
    /// The secret never leaves the engine that holds it.
    #[default]
    Local,
    /// Only the locator crosses, the remote engine resolves it with its own sources.
    Reference,
    /// The resolved value crosses.
    Value,
}

impl FromStr for SecretTransmission {
    type Err = SecretError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "local" => Ok(SecretTransmission::Local),
            "reference" => Ok(SecretTransmission::Reference),
            "value" => Ok(SecretTransmission::Value),
            other => Err(SecretError::InvalidTransmission(other.to_string())),
        }
    }
}

impl Display for SecretTransmission {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SecretTransmission::Local => write!(f, "local"),
            SecretTransmission::Reference => write!(f, "reference"),
            SecretTransmission::Value => write!(f, "value"),
        }
    }
}

/// Which elements may reveal a secret.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SecretReveal {
    #[default]
    Any,
    /// Identifiers of the elements allowed to reveal the secret, such as `std/secret::reveal`.
    Only(Vec<String>),
}

impl SecretReveal {
    /// Gives the elements allowed by both.
    pub fn narrow(&self, other: &SecretReveal) -> SecretReveal {
        match (self, other) {
            (SecretReveal::Any, other) => other.clone(),
            (me, SecretReveal::Any) => me.clone(),
            (SecretReveal::Only(mine), SecretReveal::Only(others)) => SecretReveal::Only(
                mine.iter()
                    .filter(|element| others.contains(element))
                    .cloned()
                    .collect(),
            ),
        }
    }

    pub fn allows(&self, element: &str) -> bool {
        match self {
            SecretReveal::Any => true,
            SecretReveal::Only(elements) => elements.iter().any(|allowed| allowed == element),
        }
    }
}

/// Restrictions travelling with a secret.
///
/// The default policy keeps the secret local, lets any element reveal it,
/// and refuses the plain `std/secret::reveal` treatment.
/// A policy is set when the secret is created, and can only be narrowed afterwards.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SecretPolicy {
    pub transmission: SecretTransmission,
    pub reveal: SecretReveal,
    /// Whether the plain `std/secret::reveal` treatment may reveal the secret.
    pub plain_reveal: bool,
}

impl SecretPolicy {
    /// Policy restricting nothing, neutral when narrowing.
    pub fn unrestricted() -> Self {
        Self {
            transmission: SecretTransmission::Value,
            reveal: SecretReveal::Any,
            plain_reveal: true,
        }
    }

    /// Gives the most restrictive combination of both policies.
    pub fn narrow(&self, other: &SecretPolicy) -> SecretPolicy {
        SecretPolicy {
            transmission: self.transmission.min(other.transmission),
            reveal: self.reveal.narrow(&other.reveal),
            plain_reveal: self.plain_reveal && other.plain_reveal,
        }
    }
}

/// Where the value of a secret comes from.
pub enum SecretOrigin {
    /// Resolved when revealed, such as `env:MELODIUM_SECRET_NAME`, `file:PATH` or `<source>:<path>`.
    Locator(String),
    /// Value held in memory, such as one concealed at runtime.
    Inline(Value),
    /// Computed from the values of other secrets when revealed.
    Derived {
        inputs: Vec<Secret>,
        derivation: Arc<dyn SecretDerivation>,
    },
}

impl Debug for SecretOrigin {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SecretOrigin::Locator(locator) => f.debug_tuple("Locator").field(locator).finish(),
            SecretOrigin::Inline(_) => f.debug_tuple("Inline").finish_non_exhaustive(),
            SecretOrigin::Derived { inputs, derivation } => f
                .debug_struct("Derived")
                .field("inputs", inputs)
                .field("derivation", derivation)
                .finish(),
        }
    }
}

/// What crosses to a distant engine for a secret.
///
/// Holds plaintext for a value transfer, so it has no `Debug`.
pub enum SecretTransfer {
    /// The locator, resolved by the distant engine with its own sources.
    Reference(String),
    /// The resolved value.
    Value(Value),
}

/// Value of an inline secret, wiped once the secret is dropped, or before the process exits.
struct InlineValue(Mutex<Option<Value>>);

impl Wipe for InlineValue {
    fn wipe(&self) -> bool {
        match self.0.lock().unwrap().take() {
            Some(mut value) => {
                wipe_value(&mut value);
                count_wiped();
                true
            }
            None => false,
        }
    }
}

impl Drop for InlineValue {
    fn drop(&mut self) {
        self.wipe();
    }
}

/// Origin as kept by a secret.
enum Origin {
    Locator(String),
    Inline(Arc<InlineValue>),
    Derived {
        inputs: Vec<Secret>,
        derivation: Arc<dyn SecretDerivation>,
    },
}

impl Debug for Origin {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Origin::Locator(locator) => f.debug_tuple("Locator").field(locator).finish(),
            Origin::Inline(_) => f.debug_tuple("Inline").finish_non_exhaustive(),
            Origin::Derived { inputs, derivation } => f
                .debug_struct("Derived")
                .field("inputs", inputs)
                .field("derivation", derivation)
                .finish(),
        }
    }
}

/// Splits a `<scheme>:<path>` locator, the scheme being made of ASCII letters,
/// digits, `-` and `_`, and starting with a letter.
fn split_locator(locator: &str) -> Option<(&str, &str)> {
    let (scheme, path) = locator.split_once(':')?;
    let mut chars = scheme.chars();
    if chars.next()?.is_ascii_alphabetic()
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Some((scheme, path))
    } else {
        None
    }
}

struct SecretInner {
    id: SecretId,
    name: String,
    datatype: DataType,
    policy: SecretPolicy,
    origin: Arc<Origin>,
}

/// Sensitive value, carried as `Value::Secret`.
///
/// A secret keeps where its value comes from rather than the value itself
/// (except for inline secrets), and is shared by reference: clones designate
/// the same secret, and equality compares identity.
///
/// The value is only reachable through `reveal`, which checks the policy,
/// records the access, and lends the value to a closure.
///
/// `Debug` and `Display` only show a placeholder with the name, never the value.
#[derive(Clone)]
pub struct Secret(Arc<SecretInner>);

impl Secret {
    /// Creates a secret holding a value of type `datatype`.
    ///
    /// Fails if `datatype` contains a secret, if an inline value does not match
    /// `datatype`, or if a locator is not `<scheme>:<path>`.
    /// A derived secret gets `policy` narrowed by the policies of its inputs.
    pub fn new(
        name: String,
        datatype: DataType,
        policy: SecretPolicy,
        origin: SecretOrigin,
    ) -> core::result::Result<Self, SecretError> {
        if datatype.contains_secret() {
            return Err(SecretError::NestedSecret(datatype));
        }
        let policy = match &origin {
            SecretOrigin::Inline(value) => {
                if value.datatype() != datatype {
                    return Err(SecretError::MismatchingValue);
                }
                policy
            }
            SecretOrigin::Locator(locator) => {
                if split_locator(locator).is_none() {
                    return Err(SecretError::InvalidLocator(locator.clone()));
                }
                policy
            }
            SecretOrigin::Derived { inputs, .. } => inputs
                .iter()
                .fold(policy, |policy, input| policy.narrow(input.policy())),
        };

        let origin = match origin {
            SecretOrigin::Locator(locator) => Origin::Locator(locator),
            SecretOrigin::Inline(value) => {
                let inline = Arc::new(InlineValue(Mutex::new(Some(value))));
                register_wipe(Arc::downgrade(&inline) as Weak<dyn Wipe>);
                Origin::Inline(inline)
            }
            SecretOrigin::Derived { inputs, derivation } => Origin::Derived { inputs, derivation },
        };

        Ok(Self(Arc::new(SecretInner {
            id: NEXT_SECRET_ID.fetch_add(1, Ordering::Relaxed),
            name,
            datatype,
            policy,
            origin: Arc::new(origin),
        })))
    }

    /// Creates a secret resolved from `locator`, named after it, with the default policy.
    ///
    /// This is what locator literals such as `"env:MELODIUM_SECRET_DB_PASSWORD"` give.
    pub fn from_locator(
        locator: &str,
        datatype: DataType,
    ) -> core::result::Result<Self, SecretError> {
        Self::new(
            locator.to_string(),
            datatype,
            SecretPolicy::default(),
            SecretOrigin::Locator(locator.to_string()),
        )
    }

    /// Creates a secret computed from `inputs` by `derivation` when revealed.
    ///
    /// Its policy is the most restrictive combination of the policies of `inputs`,
    /// or the default policy if there is no input.
    pub fn derive(
        name: String,
        datatype: DataType,
        inputs: Vec<Secret>,
        derivation: Arc<dyn SecretDerivation>,
    ) -> core::result::Result<Self, SecretError> {
        let policy = if inputs.is_empty() {
            SecretPolicy::default()
        } else {
            SecretPolicy::unrestricted()
        };
        Self::new(
            name,
            datatype,
            policy,
            SecretOrigin::Derived { inputs, derivation },
        )
    }

    pub fn id(&self) -> SecretId {
        self.0.id
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// Type of the value held, `T` for a `Secret<T>`.
    pub fn datatype(&self) -> &DataType {
        &self.0.datatype
    }

    pub fn policy(&self) -> &SecretPolicy {
        &self.0.policy
    }

    /// Locator of the secret, if its value is resolved from a source.
    pub fn locator(&self) -> Option<&str> {
        match &*self.0.origin {
            Origin::Locator(locator) => Some(locator),
            Origin::Inline(_) | Origin::Derived { .. } => None,
        }
    }

    /// Gives the secrets a derived secret is computed from, at any depth.
    pub fn derived_from(&self) -> Vec<Secret> {
        let mut secrets = Vec::new();
        if let Origin::Derived { inputs, .. } = &*self.0.origin {
            for input in inputs {
                secrets.push(input.clone());
                secrets.extend(input.derived_from());
            }
        }
        secrets
    }

    /// Gives a secret with the same name and origin, and a policy narrowed by `policy`.
    pub fn narrow(&self, policy: &SecretPolicy) -> Secret {
        Self(Arc::new(SecretInner {
            id: NEXT_SECRET_ID.fetch_add(1, Ordering::Relaxed),
            name: self.0.name.clone(),
            datatype: self.0.datatype.clone(),
            policy: self.0.policy.narrow(policy),
            origin: Arc::clone(&self.0.origin),
        }))
    }

    /// Gives what crosses to a distant engine for the secret, following its transmission
    /// policy, for the element `access` belongs to, as `SecretsAccess` does:
    /// - `local` secrets are refused;
    /// - `reference` secrets give their locator, those without locator are refused;
    /// - `value` secrets give their resolved value, over an encrypted connection only.
    ///
    /// Transmissions and refusals are recorded, refusals also in the log.
    pub(crate) async fn transmit(
        &self,
        access: &SecretsAccess,
        encrypted: bool,
    ) -> core::result::Result<SecretTransfer, SecretError> {
        let host = access.host.upgrade().ok_or(SecretError::WorldEnded)?;
        let audit = |outcome| SecretAudit {
            secret_id: self.0.id,
            secret_name: self.0.name.clone(),
            element: access.element.clone(),
            label: access.label.clone(),
            track_id: access.track_id,
            outcome,
        };

        let transmission = self.0.policy.transmission;
        let denial = match transmission {
            _ if !host.revealing() => Some("secrets are only sent once every model is initialized"),
            SecretTransmission::Local => Some("its policy keeps it on this engine"),
            SecretTransmission::Reference if self.locator().is_none() => {
                Some("it has no locator to send by reference")
            }
            SecretTransmission::Value if !encrypted => {
                Some("its value can only be sent over an encrypted connection")
            }
            _ => None,
        };
        if let Some(reason) = denial {
            host.secret_audit(audit(SecretAuditOutcome::Denied(reason.to_string())))
                .await;
            return Err(SecretError::Denied(reason.to_string()));
        }

        let transfer = match self.locator() {
            Some(locator) if transmission == SecretTransmission::Reference => {
                SecretTransfer::Reference(locator.to_string())
            }
            _ => match self.resolve(&*host, true).await {
                Ok(value) => {
                    host.add_masked_value(&self.0.name, &value);
                    SecretTransfer::Value(value)
                }
                Err(error) => {
                    host.secret_audit(audit(SecretAuditOutcome::ResolveFailed(error.clone())))
                        .await;
                    return Err(SecretError::ResolveFailed(error));
                }
            },
        };
        host.secret_audit(audit(SecretAuditOutcome::Transmitted(transmission)))
            .await;
        Ok(transfer)
    }

    /// Resolves the value of the secret and drops it right away.
    pub(crate) async fn check_resolution(
        &self,
        host: &dyn SecretsHost,
    ) -> core::result::Result<(), SecretError> {
        let mut value = self
            .resolve(host, false)
            .await
            .map_err(SecretError::ResolveFailed)?;
        wipe_value(&mut value);
        Ok(())
    }

    /// Lends the value of the secret to `f`, for the element `access` belongs to,
    /// as `SecretsAccess` does.
    pub(crate) async fn access<R>(
        &self,
        access: &SecretsAccess,
        plain: bool,
        f: impl FnOnce(&Value) -> R + Send,
    ) -> core::result::Result<R, SecretError> {
        let host = access.host.upgrade().ok_or(SecretError::WorldEnded)?;
        let audit_of = |secret: &Secret, outcome| SecretAudit {
            secret_id: secret.0.id,
            secret_name: secret.0.name.clone(),
            element: access.element.clone(),
            label: access.label.clone(),
            track_id: access.track_id,
            outcome,
        };
        let audit = |outcome| audit_of(self, outcome);

        let element = access.element.to_string();
        let denial = if !host.revealing() {
            Some("secrets are only revealed once every model is initialized".to_string())
        } else if !self.0.policy.reveal.allows(&element) {
            Some(format!("its policy does not allow {element} to reveal it"))
        } else if plain && !self.0.policy.plain_reveal {
            Some("its policy does not allow plain reveal".to_string())
        } else if let Some(scheme) = plain.then(|| self.plain_reveal_refusal(&*host)).flatten() {
            Some(format!(
                "secrets from '{scheme}:' cannot be plainly revealed"
            ))
        } else {
            None
        };
        if let Some(reason) = denial {
            host.secret_audit(audit(SecretAuditOutcome::Denied(reason.clone())))
                .await;
            return Err(SecretError::Denied(reason));
        }

        match self.resolve(&*host, true).await {
            Ok(mut value) => {
                host.add_masked_value(&self.0.name, &value);
                // The value of a derived secret exposes the values it is computed from.
                for input in self.derived_from() {
                    host.secret_audit(audit_of(&input, SecretAuditOutcome::Revealed))
                        .await;
                }
                host.secret_audit(audit(SecretAuditOutcome::Revealed)).await;
                let result = f(&value);
                wipe_value(&mut value);
                Ok(result)
            }
            Err(error) => {
                host.secret_audit(audit(SecretAuditOutcome::ResolveFailed(error.clone())))
                    .await;
                Err(SecretError::ResolveFailed(error))
            }
        }
    }

    /// Gives the scheme of the source of this secret if that source refuses plain reveal,
    /// or of the first such source a derived secret comes from.
    fn plain_reveal_refusal(&self, host: &dyn SecretsHost) -> Option<String> {
        match &*self.0.origin {
            Origin::Inline(_) => None,
            Origin::Locator(locator) => {
                let (scheme, _) = split_locator(locator)?;
                let source = host.secret_source(scheme)?;
                (!source.plain_reveal()).then(|| scheme.to_string())
            }
            Origin::Derived { inputs, .. } => inputs
                .iter()
                .find_map(|input| input.plain_reveal_refusal(host)),
        }
    }

    /// Gives the scheme the secret is located in, if it has a locator.
    pub fn scheme(&self) -> Option<&str> {
        self.locator()
            .and_then(split_locator)
            .map(|(scheme, _)| scheme)
    }

    /// Resolves the value of the secret.
    ///
    /// When `revealing`, the values of the secrets it is derived from are registered
    /// for masking, the caller having to register the resolved value.
    fn resolve<'a>(
        &'a self,
        host: &'a dyn SecretsHost,
        revealing: bool,
    ) -> Pin<Box<dyn Future<Output = core::result::Result<Value, String>> + Send + 'a>> {
        Box::pin(async move {
            let value = match &*self.0.origin {
                Origin::Inline(inline) => inline
                    .0
                    .lock()
                    .unwrap()
                    .clone()
                    .ok_or_else(|| "the value was wiped".to_string())?,
                Origin::Locator(locator) => {
                    let (scheme, path) = split_locator(locator)
                        .ok_or_else(|| format!("'{locator}' is not a locator"))?;
                    let source = host
                        .secret_source(scheme)
                        .ok_or_else(|| format!("no secret source '{scheme}' for '{locator}'"))?;
                    source.resolve(path, &self.0.datatype).await?
                }
                Origin::Derived { inputs, derivation } => {
                    let mut values = Vec::with_capacity(inputs.len());
                    let mut failure = None;
                    for input in inputs {
                        match input.resolve(host, revealing).await {
                            Ok(value) => {
                                if revealing {
                                    host.add_masked_value(input.name(), &value);
                                }
                                values.push(value);
                            }
                            Err(error) => {
                                failure = Some(format!("{input}: {error}"));
                                break;
                            }
                        }
                    }
                    let derived = match failure {
                        Some(failure) => Err(failure),
                        None => derivation.derive(&values),
                    };
                    values.iter_mut().for_each(wipe_value);
                    derived?
                }
            };
            if value.datatype() == self.0.datatype {
                Ok(value)
            } else {
                let mut value = value;
                wipe_value(&mut value);
                Err(format!("{self} did not get a {} value", self.0.datatype))
            }
        })
    }
}

impl PartialEq for Secret {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Secret {}

impl Debug for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Secret")
            .field("id", &self.0.id)
            .field("name", &self.0.name)
            .field("datatype", &self.0.datatype)
            .field("policy", &self.0.policy)
            .finish_non_exhaustive()
    }
}

impl Display for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "<secret {:?}>", self.0.name)
    }
}

/// Refused, except inside data values crossing to a distant engine, where the secret wire
/// set by the distribution layer writes it following its transmission policy.
impl serde::Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        current_secret_wire(|wire| match wire.map(|wire| wire.write(self)) {
            Some(Ok(written)) => erased_serde::serialize(&*written, serializer),
            Some(Err(error)) => Err(<S::Error as serde::ser::Error>::custom(error)),
            None => Err(<S::Error as serde::ser::Error>::custom(format!(
                "{self} cannot be serialized"
            ))),
        })
    }
}

/// Refused, except inside data values received from a distant engine.
impl<'de> serde::Deserialize<'de> for Secret {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        current_secret_wire(|wire| match wire {
            Some(wire) => wire
                .read(&mut <dyn erased_serde::Deserializer>::erase(deserializer))
                .map_err(<D::Error as serde::de::Error>::custom),
            None => Err(<D::Error as serde::de::Error>::custom(
                "secrets can only be received from a distant engine",
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: &str = "s3cr3t-sentinel-value";

    fn inline_secret() -> Secret {
        Secret::new(
            "db_password".to_string(),
            DataType::String,
            SecretPolicy::default(),
            SecretOrigin::Inline(Value::String(SENTINEL.to_string())),
        )
        .unwrap()
    }

    #[test]
    fn default_policy_is_local_any_without_plain_reveal() {
        let policy = SecretPolicy::default();
        assert_eq!(policy.transmission, SecretTransmission::Local);
        assert_eq!(policy.reveal, SecretReveal::Any);
        assert!(!policy.plain_reveal);
    }

    #[test]
    fn display_shows_placeholder_with_name_only() {
        let secret = inline_secret();
        assert_eq!(secret.to_string(), r#"<secret "db_password">"#);
        assert_eq!(
            Value::Secret(secret).to_string(),
            r#"<secret "db_password">"#
        );
    }

    #[test]
    fn debug_never_shows_inline_value() {
        let secret = inline_secret();
        let value = Value::Secret(secret.clone());
        for rendered in [
            format!("{secret:?}"),
            format!("{secret:#?}"),
            format!("{value:?}"),
            format!("{value:#?}"),
            format!("{:?}", Value::Vec(vec![value.clone()])),
            format!("{:?}", Value::Option(Some(Box::new(value.clone())))),
            format!(
                "{:?}",
                SecretOrigin::Inline(Value::String(SENTINEL.to_string()))
            ),
        ] {
            assert!(!rendered.contains(SENTINEL), "leaked in {}", rendered);
        }
        assert!(format!("{secret:?}").contains("db_password"));
    }

    #[test]
    fn runtime_display_never_shows_inline_value() {
        struct RuntimeDisplay<'a>(&'a Value);
        impl Display for RuntimeDisplay<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                crate::executive::DataTrait::display(self.0, f)
            }
        }

        let value = Value::Secret(inline_secret());
        assert_eq!(
            RuntimeDisplay(&value).to_string(),
            r#"<secret "db_password">"#
        );
    }

    #[test]
    fn serialization_is_refused() {
        use crate::executive::DataTrait;

        let value = Value::Secret(inline_secret());
        for value in [
            value.clone(),
            Value::Vec(vec![value.clone()]),
            Value::Option(Some(Box::new(value.clone()))),
        ] {
            let error = serde_json::to_string(&value).unwrap_err().to_string();
            assert!(error.contains("db_password"), "{}", error);
            assert!(!error.contains(SENTINEL), "leaked in {}", error);

            let error = serde_json::to_string(&value as &dyn DataTrait)
                .unwrap_err()
                .to_string();
            assert!(error.contains("db_password"), "{}", error);
            assert!(!error.contains(SENTINEL), "leaked in {}", error);
        }

        let mut output = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut output);
        let mut serializer = <dyn erased_serde::Serializer>::erase(&mut serializer);
        assert!(DataTrait::serialize(&value, &mut serializer).is_err());
        drop(serializer);
        assert!(output.is_empty());
    }

    #[test]
    fn secrets_inside_data_only_cross_through_a_wire() {
        use crate::executive::{with_secret_wire, SecretWire};

        #[derive(serde::Serialize, serde::Deserialize)]
        struct Holder {
            token: Secret,
        }

        /// Writes the name, and reads it back as an `env:` locator.
        struct NameWire;
        impl SecretWire for NameWire {
            fn write(
                &self,
                secret: &Secret,
            ) -> core::result::Result<Box<dyn erased_serde::Serialize + '_>, String> {
                Ok(Box::new(secret.name().to_string()))
            }
            fn read<'de>(
                &self,
                deserializer: &mut dyn erased_serde::Deserializer<'de>,
            ) -> core::result::Result<Secret, String> {
                let name: String =
                    erased_serde::deserialize(deserializer).map_err(|e| e.to_string())?;
                Ok(Secret::from_locator(&format!("env:{name}"), DataType::String).unwrap())
            }
        }

        let holder = Holder {
            token: inline_secret(),
        };
        let error = serde_json::to_string(&holder).unwrap_err().to_string();
        assert!(!error.contains(SENTINEL), "leaked in {}", error);

        let wire: Box<dyn SecretWire> = Box::new(NameWire);
        let json = with_secret_wire(&wire, || serde_json::to_string(&holder)).unwrap();
        assert_eq!(json, r#"{"token":"db_password"}"#);

        assert!(serde_json::from_str::<Holder>(&json).is_err());
        let received: Holder = with_secret_wire(&wire, || serde_json::from_str(&json)).unwrap();
        assert_eq!(received.token.locator(), Some("env:db_password"));
    }

    #[test]
    fn equality_is_identity() {
        let first = inline_secret();
        let second = inline_secret();
        assert_eq!(first, first.clone());
        assert_ne!(first, second);
        assert_ne!(first.id(), second.id());
        assert_eq!(Value::Secret(first.clone()), Value::Secret(first.clone()));
        assert_ne!(Value::Secret(first), Value::Secret(second));
    }

    #[test]
    fn datatype_is_always_secret_of_inner() {
        let secret = Secret::new(
            "token".to_string(),
            DataType::Vec(Box::new(DataType::Byte)),
            SecretPolicy::default(),
            SecretOrigin::Locator("env:TOKEN".to_string()),
        )
        .unwrap();
        let datatype = Value::Secret(secret).datatype();
        assert!(matches!(
            &datatype,
            DataType::Secret(inner) if matches!(inner.as_ref(), DataType::Vec(byte) if matches!(byte.as_ref(), DataType::Byte))
        ));
        assert_ne!(datatype, DataType::Vec(Box::new(DataType::Byte)));
        assert_ne!(
            datatype,
            DataType::Secret(Box::new(DataType::Vec(Box::new(DataType::U8))))
        );
    }

    #[test]
    fn estimated_size_does_not_depend_on_inline_value() {
        let short = Secret::new(
            "short".to_string(),
            DataType::String,
            SecretPolicy::default(),
            SecretOrigin::Inline(Value::String("a".to_string())),
        )
        .unwrap();
        let long = Secret::new(
            "long".to_string(),
            DataType::String,
            SecretPolicy::default(),
            SecretOrigin::Inline(Value::String("a".repeat(4096))),
        )
        .unwrap();
        assert_eq!(
            Value::Secret(short).estimated_size(),
            Value::Secret(long).estimated_size()
        );
    }

    #[test]
    fn locator_is_only_given_for_locator_origin() {
        let secret = Secret::new(
            "token".to_string(),
            DataType::String,
            SecretPolicy::default(),
            SecretOrigin::Locator("env:TOKEN".to_string()),
        )
        .unwrap();
        assert_eq!(secret.locator(), Some("env:TOKEN"));
        assert_eq!(inline_secret().locator(), None);
    }

    #[test]
    fn contains_secret_looks_through_containers() {
        let secret = Value::Secret(inline_secret());
        assert!(secret.contains_secret());
        assert!(
            Value::Vec(vec![Value::String("plain".to_string()), secret.clone()]).contains_secret()
        );
        assert!(Value::Option(Some(Box::new(Value::Vec(vec![secret])))).contains_secret());
        assert!(!Value::Vec(vec![Value::String(SENTINEL.to_string())]).contains_secret());
        assert!(!Value::Option(None).contains_secret());
    }

    #[test]
    fn nested_secrets_are_refused() {
        let nested = DataType::Secret(Box::new(DataType::String));
        for datatype in [
            nested.clone(),
            DataType::Vec(Box::new(nested.clone())),
            DataType::Option(Box::new(nested)),
        ] {
            assert!(Secret::new(
                "nested".to_string(),
                datatype,
                SecretPolicy::default(),
                SecretOrigin::Locator("env:NESTED".to_string()),
            )
            .is_err());
        }
    }

    #[test]
    fn narrowing_gives_the_most_restrictive_policy() {
        let wide = SecretPolicy {
            transmission: SecretTransmission::Value,
            reveal: SecretReveal::Only(vec!["a::A".to_string(), "b::B".to_string()]),
            plain_reveal: true,
        };
        let narrow = SecretPolicy {
            transmission: SecretTransmission::Reference,
            reveal: SecretReveal::Only(vec!["b::B".to_string(), "c::C".to_string()]),
            plain_reveal: false,
        };
        let expected = SecretPolicy {
            transmission: SecretTransmission::Reference,
            reveal: SecretReveal::Only(vec!["b::B".to_string()]),
            plain_reveal: false,
        };
        assert_eq!(wide.narrow(&narrow), expected);
        assert_eq!(narrow.narrow(&wide), expected);
        assert_eq!(wide.narrow(&SecretPolicy::unrestricted()), wide);
        assert_eq!(
            SecretPolicy::unrestricted().narrow(&SecretPolicy::default()),
            SecretPolicy::default()
        );
        assert_eq!(
            SecretPolicy::default().narrow(&SecretPolicy::unrestricted()),
            SecretPolicy::default()
        );

        assert!(SecretReveal::Any.allows("a::A"));
        assert!(expected.reveal.allows("b::B"));
        assert!(!expected.reveal.allows("a::A"));
    }

    #[test]
    fn transmissions_parse_from_their_names() {
        for transmission in [
            SecretTransmission::Local,
            SecretTransmission::Reference,
            SecretTransmission::Value,
        ] {
            assert_eq!(
                transmission.to_string().parse::<SecretTransmission>(),
                Ok(transmission)
            );
        }
        assert_eq!(
            "remote".parse::<SecretTransmission>(),
            Err(SecretError::InvalidTransmission("remote".to_string()))
        );
    }

    #[test]
    fn narrowed_secrets_keep_name_and_origin() {
        let secret = Secret::from_locator("file:/run/secrets/db", DataType::String).unwrap();
        let narrowed = secret.narrow(&SecretPolicy {
            reveal: SecretReveal::Only(vec!["sql/pool::SqlPool".to_string()]),
            ..SecretPolicy::unrestricted()
        });
        assert_ne!(narrowed, secret);
        assert_ne!(narrowed.id(), secret.id());
        assert_eq!(narrowed.name(), secret.name());
        assert_eq!(narrowed.locator(), Some("file:/run/secrets/db"));
        assert_eq!(
            narrowed.policy().reveal,
            SecretReveal::Only(vec!["sql/pool::SqlPool".to_string()])
        );
        assert_eq!(narrowed.policy().transmission, SecretTransmission::Local);
    }

    #[test]
    fn locators_need_a_scheme() {
        for locator in [
            "env:NAME",
            "file:/run/secrets/db",
            "vault:db/password#key",
            "my-source_2:x",
            "env:",
        ] {
            let secret = Secret::from_locator(locator, DataType::String).unwrap();
            assert_eq!(secret.name(), locator);
            assert_eq!(secret.locator(), Some(locator));
            assert_eq!(secret.policy(), &SecretPolicy::default());
        }
        for locator in ["hunter2", ":path", "1env:NAME", "en v:NAME", ""] {
            assert_eq!(
                Secret::from_locator(locator, DataType::String),
                Err(SecretError::InvalidLocator(locator.to_string()))
            );
        }
    }

    #[derive(Debug)]
    struct Concatenation;

    impl SecretDerivation for Concatenation {
        fn derive(&self, inputs: &[Value]) -> core::result::Result<Value, String> {
            Ok(Value::String(
                inputs
                    .iter()
                    .map(|value| match value {
                        Value::String(text) => text.as_str(),
                        _ => "",
                    })
                    .collect(),
            ))
        }
    }

    #[test]
    fn derived_secrets_combine_input_policies() {
        let open = Secret::new(
            "open".to_string(),
            DataType::String,
            SecretPolicy {
                transmission: SecretTransmission::Value,
                reveal: SecretReveal::Only(vec!["a::A".to_string(), "b::B".to_string()]),
                plain_reveal: true,
            },
            SecretOrigin::Locator("env:OPEN".to_string()),
        )
        .unwrap();
        let closed = Secret::new(
            "closed".to_string(),
            DataType::String,
            SecretPolicy {
                transmission: SecretTransmission::Reference,
                reveal: SecretReveal::Only(vec!["b::B".to_string()]),
                plain_reveal: true,
            },
            SecretOrigin::Inline(Value::String(SENTINEL.to_string())),
        )
        .unwrap();

        let derived = Secret::derive(
            "derived".to_string(),
            DataType::String,
            vec![open.clone(), closed.clone()],
            Arc::new(Concatenation),
        )
        .unwrap();
        assert_eq!(
            derived.policy(),
            &SecretPolicy {
                transmission: SecretTransmission::Reference,
                reveal: SecretReveal::Only(vec!["b::B".to_string()]),
                plain_reveal: true,
            }
        );
        assert_eq!(derived.name(), "derived");
        assert_eq!(derived.locator(), None);

        // A policy given with a derived origin cannot be wider than the inputs ones.
        let widened = Secret::new(
            "widened".to_string(),
            DataType::String,
            SecretPolicy::unrestricted(),
            SecretOrigin::Derived {
                inputs: vec![closed.clone()],
                derivation: Arc::new(Concatenation),
            },
        )
        .unwrap();
        assert_eq!(widened.policy(), closed.policy());

        let nested = Secret::derive(
            "nested".to_string(),
            DataType::String,
            vec![derived.clone(), open.clone()],
            Arc::new(Concatenation),
        )
        .unwrap();
        assert_eq!(
            nested.derived_from(),
            vec![derived, open.clone(), closed, open]
        );
        let debug = format!("{nested:?} {:?}", nested.0.origin);
        assert!(!debug.contains(SENTINEL), "leaked in {}", debug);
    }

    #[test]
    fn derived_secrets_without_inputs_get_the_default_policy() {
        let derived = Secret::derive(
            "constant".to_string(),
            DataType::String,
            Vec::new(),
            Arc::new(Concatenation),
        )
        .unwrap();
        assert_eq!(derived.policy(), &SecretPolicy::default());
        assert_eq!(
            Secret::derive(
                "nested".to_string(),
                DataType::Secret(Box::new(DataType::String)),
                Vec::new(),
                Arc::new(Concatenation),
            ),
            Err(SecretError::NestedSecret(DataType::Secret(Box::new(
                DataType::String
            ))))
        );
    }

    #[test]
    fn inline_values_are_wiped_when_dropped_or_asked() {
        let wiped = crate::executive::wiped_count();
        let dropped = inline_secret();
        let narrowed = dropped.narrow(&SecretPolicy::unrestricted());
        drop(dropped);
        // Still held by the narrowed secret, sharing the origin.
        drop(narrowed);
        assert!(crate::executive::wiped_count() > wiped);

        let kept = inline_secret();
        match &*kept.0.origin {
            Origin::Inline(inline) => {
                assert!(inline.wipe());
                assert!(inline.0.lock().unwrap().is_none());
                assert!(!inline.wipe());
            }
            _ => panic!("inline origin expected"),
        }
    }

    #[test]
    fn inline_value_must_match_datatype() {
        assert!(Secret::new(
            "mismatch".to_string(),
            DataType::String,
            SecretPolicy::default(),
            SecretOrigin::Inline(Value::U64(42)),
        )
        .is_err());
    }
}
