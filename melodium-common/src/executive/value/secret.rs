use super::Value;
use crate::descriptor::DataType;
use crate::executive::{
    PackedArray, SecretAccess, SecretAudit, SecretAuditOutcome, SecretError, World,
};
use core::fmt::{Debug, Display, Formatter, Result};
use core::str::FromStr;
use core::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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
    /// Resolved when revealed, such as `env:NAME`, `file:PATH` or `<source>:<path>`.
    Locator(String),
    /// Value held in memory, such as one concealed at runtime.
    Inline(Value),
}

impl Debug for SecretOrigin {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SecretOrigin::Locator(locator) => f.debug_tuple("Locator").field(locator).finish(),
            SecretOrigin::Inline(_) => f.debug_tuple("Inline").finish_non_exhaustive(),
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
    origin: Arc<SecretOrigin>,
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
    pub fn new(
        name: String,
        datatype: DataType,
        policy: SecretPolicy,
        origin: SecretOrigin,
    ) -> core::result::Result<Self, SecretError> {
        if datatype.contains_secret() {
            return Err(SecretError::NestedSecret(datatype));
        }
        match &origin {
            SecretOrigin::Inline(value) => {
                if value.datatype() != datatype {
                    return Err(SecretError::MismatchingValue);
                }
            }
            SecretOrigin::Locator(locator) => {
                if split_locator(locator).is_none() {
                    return Err(SecretError::InvalidLocator(locator.clone()));
                }
            }
        }

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
    /// This is what locator literals such as `"env:DB_PASSWORD"` give.
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
            SecretOrigin::Locator(locator) => Some(locator),
            SecretOrigin::Inline(_) => None,
        }
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

    /// Lends the value of the secret to `f`, for the element designated by `access`.
    ///
    /// The policy is checked, the value resolved, and the access recorded on the world:
    /// denials and resolution failures are also logged as errors.
    /// The value is dropped once `f` returns.
    pub async fn reveal<R>(
        &self,
        access: &SecretAccess,
        f: impl FnOnce(&Value) -> R + Send,
    ) -> core::result::Result<R, SecretError> {
        self.access(access, false, f).await
    }

    /// Same as `reveal`, for a `Secret<string>`.
    pub async fn reveal_str<R>(
        &self,
        access: &SecretAccess,
        f: impl FnOnce(&str) -> R + Send,
    ) -> core::result::Result<R, SecretError> {
        self.reveal(access, |value| match value {
            Value::String(value) => Ok(f(value)),
            _ => Err(SecretError::MismatchingValue),
        })
        .await?
    }

    /// Same as `reveal`, for a `Secret<Vec<byte>>`.
    pub async fn reveal_bytes<R>(
        &self,
        access: &SecretAccess,
        f: impl FnOnce(&[u8]) -> R + Send,
    ) -> core::result::Result<R, SecretError> {
        self.reveal(access, |value| match value {
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

    /// Lends the value of the secret to `f`, for the plain `std/secret::reveal` treatment.
    ///
    /// Same as `reveal`, but also requires the policy to allow plain reveal.
    pub async fn reveal_plainly<R>(
        &self,
        access: &SecretAccess,
        f: impl FnOnce(&Value) -> R + Send,
    ) -> core::result::Result<R, SecretError> {
        self.access(access, true, f).await
    }

    /// Resolves the value of the secret and drops it right away,
    /// to check that the secret can be resolved.
    ///
    /// The value is not given to anyone, so the access is not recorded.
    pub async fn check_resolution(
        &self,
        world: &Arc<dyn World>,
    ) -> core::result::Result<(), SecretError> {
        match &*self.0.origin {
            SecretOrigin::Inline(_) => Ok(()),
            SecretOrigin::Locator(locator) => self
                .resolve(world, locator)
                .await
                .map(|_| ())
                .map_err(SecretError::ResolveFailed),
        }
    }

    async fn access<R>(
        &self,
        access: &SecretAccess,
        plain: bool,
        f: impl FnOnce(&Value) -> R + Send,
    ) -> core::result::Result<R, SecretError> {
        let world = access.world().ok_or(SecretError::WorldEnded)?;
        let audit = |outcome| SecretAudit {
            secret_id: self.0.id,
            secret_name: self.0.name.clone(),
            element: access.element().clone(),
            label: access.label().map(str::to_string),
            track_id: access.track_id(),
            outcome,
        };

        let element = access.element().to_string();
        let denial = if !self.0.policy.reveal.allows(&element) {
            Some(format!("its policy does not allow {element} to reveal it"))
        } else if plain && !self.0.policy.plain_reveal {
            Some("its policy does not allow plain reveal".to_string())
        } else if let Some(scheme) = plain.then(|| self.plain_reveal_refusal(&world)).flatten() {
            Some(format!(
                "secrets from '{scheme}:' cannot be plainly revealed"
            ))
        } else {
            None
        };
        if let Some(reason) = denial {
            world
                .secret_audit(audit(SecretAuditOutcome::Denied(reason.clone())))
                .await;
            return Err(SecretError::Denied(reason));
        }

        match &*self.0.origin {
            SecretOrigin::Inline(value) => {
                world
                    .secret_audit(audit(SecretAuditOutcome::Revealed))
                    .await;
                Ok(f(value))
            }
            SecretOrigin::Locator(locator) => match self.resolve(&world, locator).await {
                Ok(value) => {
                    world
                        .secret_audit(audit(SecretAuditOutcome::Revealed))
                        .await;
                    Ok(f(&value))
                }
                Err(error) => {
                    world
                        .secret_audit(audit(SecretAuditOutcome::ResolveFailed(error.clone())))
                        .await;
                    Err(SecretError::ResolveFailed(error))
                }
            },
        }
    }

    /// Gives the scheme of the source of this secret if that source refuses plain reveal.
    fn plain_reveal_refusal(&self, world: &Arc<dyn World>) -> Option<String> {
        match &*self.0.origin {
            SecretOrigin::Inline(_) => None,
            SecretOrigin::Locator(locator) => {
                let (scheme, _) = split_locator(locator)?;
                let source = world.secret_source(scheme)?;
                (!source.plain_reveal()).then(|| scheme.to_string())
            }
        }
    }

    async fn resolve(
        &self,
        world: &Arc<dyn World>,
        locator: &str,
    ) -> core::result::Result<Value, String> {
        let (scheme, path) =
            split_locator(locator).ok_or_else(|| format!("'{locator}' is not a locator"))?;
        let source = world
            .secret_source(scheme)
            .ok_or_else(|| format!("no secret source '{scheme}' for '{locator}'"))?;
        let value = source.resolve(path, &self.0.datatype).await?;
        if value.datatype() == self.0.datatype {
            Ok(value)
        } else {
            Err(format!(
                "source '{scheme}' did not give a {} value for '{locator}'",
                self.0.datatype
            ))
        }
    }
}

impl PartialEq for Secret {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

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
