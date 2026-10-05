use super::Value;
use crate::descriptor::DataType;
use core::fmt::{Debug, Display, Formatter, Result};
use core::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub type SecretId = u64;

static NEXT_SECRET_ID: AtomicU64 = AtomicU64::new(1);

/// Where a secret may go when a program is distributed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SecretTransmission {
    /// The secret never leaves the engine that holds it.
    #[default]
    Local,
    /// Only the locator crosses, the remote engine resolves it with its own sources.
    Reference,
    /// The resolved value crosses.
    Value,
}

/// Which elements may reveal a secret.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SecretReveal {
    #[default]
    Any,
    /// Identifiers of the elements allowed to reveal the secret.
    Only(Vec<String>),
}

/// Restrictions travelling with a secret.
///
/// The default policy keeps the secret local, lets any element reveal it,
/// and refuses the plain `std/secret::reveal` treatment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SecretPolicy {
    pub transmission: SecretTransmission,
    pub reveal: SecretReveal,
    /// Whether the plain `std/secret::reveal` treatment may reveal the secret.
    pub plain_reveal: bool,
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

struct SecretInner {
    id: SecretId,
    name: String,
    datatype: DataType,
    policy: SecretPolicy,
    origin: SecretOrigin,
}

/// Sensitive value, carried as `Value::Secret`.
///
/// A secret keeps where its value comes from rather than the value itself
/// (except for inline secrets), and is shared by reference: clones designate
/// the same secret, and equality compares identity.
///
/// `Debug` and `Display` only show a placeholder with the name, never the value.
#[derive(Clone)]
pub struct Secret(Arc<SecretInner>);

impl Secret {
    /// Creates a secret holding a value of type `datatype`.
    ///
    /// Returns `None` if `datatype` contains a secret, or if an inline value
    /// does not match `datatype`.
    pub fn new(
        name: String,
        datatype: DataType,
        policy: SecretPolicy,
        origin: SecretOrigin,
    ) -> Option<Self> {
        if datatype.contains_secret() {
            return None;
        }
        if let SecretOrigin::Inline(value) = &origin {
            if value.datatype() != datatype {
                return None;
            }
        }

        Some(Self(Arc::new(SecretInner {
            id: NEXT_SECRET_ID.fetch_add(1, Ordering::Relaxed),
            name,
            datatype,
            policy,
            origin,
        })))
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
        match &self.0.origin {
            SecretOrigin::Locator(locator) => Some(locator),
            SecretOrigin::Inline(_) => None,
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
            .is_none());
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
        .is_none());
    }
}
