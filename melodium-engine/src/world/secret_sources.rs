use async_trait::async_trait;
use melodium_common::descriptor::DataType;
use melodium_common::executive::{PackedArray, SecretSource, Value};
use std::sync::Arc;

/// Gives the value of a text source as `datatype`, if it is `string` or `Vec<byte>`.
#[cfg(feature = "environment")]
fn text_value(text: String, datatype: &DataType, origin: &str) -> Result<Value, String> {
    match datatype {
        DataType::String => Ok(Value::String(text)),
        DataType::Vec(inner) if matches!(**inner, DataType::Byte) => Ok(Value::Packed(
            PackedArray::Byte(Arc::new(text.into_bytes())),
        )),
        other => Err(format!("{origin} cannot be given as {other}")),
    }
}

/// Prefix of the environment variables given to programs, as secrets.
#[cfg(feature = "environment")]
pub const ENVIRONMENT_SECRET_PREFIX: &str = "MELODIUM_SECRET_";

/// `env:NAME` source, giving the value of an environment variable whose name starts with
/// `MELODIUM_SECRET_`, so that only variables set for programs on purpose are given to
/// them, and not the other ones of the host, such as the configuration of Mélodium.
///
/// Its secrets are never plainly revealed: a program only gets environment variables as
/// secrets, revealed by the elements using them.
#[cfg(feature = "environment")]
#[derive(Debug)]
pub struct EnvironmentSource;

#[cfg(feature = "environment")]
#[async_trait]
impl SecretSource for EnvironmentSource {
    fn plain_reveal(&self) -> bool {
        false
    }

    async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String> {
        if !path.starts_with(ENVIRONMENT_SECRET_PREFIX) {
            return Err(format!(
                "environment variable '{path}' is not given to programs, only the ones starting with '{ENVIRONMENT_SECRET_PREFIX}' are"
            ));
        }
        match std::env::var(path) {
            Ok(text) => text_value(text, datatype, &format!("environment variable '{path}'")),
            Err(std::env::VarError::NotPresent) => {
                Err(format!("environment variable '{path}' is not set"))
            }
            Err(std::env::VarError::NotUnicode(_)) => Err(format!(
                "environment variable '{path}' is not valid unicode"
            )),
        }
    }
}

/// `file:PATH` source, giving the content of a file.
///
/// As `Secret<string>`, one trailing newline is removed, as files are usually written
/// with one. As `Secret<Vec<byte>>`, the content is given unchanged.
///
/// Its secrets are never plainly revealed: a program only gets files as secrets, revealed
/// by the elements using them.
#[cfg(feature = "filesystem")]
#[derive(Debug)]
pub struct FileSource;

#[cfg(feature = "filesystem")]
#[async_trait]
impl SecretSource for FileSource {
    fn plain_reveal(&self) -> bool {
        false
    }

    #[cfg(not(target_os = "unknown"))]
    async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String> {
        let content = async_std::fs::read(path)
            .await
            .map_err(|err| format!("file '{path}' cannot be read: {}", err.kind()))?;
        match datatype {
            DataType::Vec(inner) if matches!(**inner, DataType::Byte) => {
                Ok(Value::Packed(PackedArray::Byte(Arc::new(content))))
            }
            DataType::String => {
                let mut text = String::from_utf8(content)
                    .map_err(|_| format!("file '{path}' is not valid UTF-8"))?;
                if text.ends_with('\n') {
                    text.pop();
                    if text.ends_with('\r') {
                        text.pop();
                    }
                }
                Ok(Value::String(text))
            }
            other => Err(format!("file '{path}' cannot be given as {other}")),
        }
    }

    #[cfg(target_os = "unknown")]
    async fn resolve(&self, path: &str, _datatype: &DataType) -> Result<Value, String> {
        Err(format!("file '{path}' cannot be read on this platform"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "filesystem")]
    #[test]
    fn files_lose_one_trailing_newline_as_strings_only() {
        let path = std::env::temp_dir().join(format!(
            "melodium_secret_sources_{}_file",
            std::process::id()
        ));
        std::fs::write(&path, "sentinel\n").unwrap();
        let path = path.to_string_lossy().to_string();

        let text = async_std::task::block_on(FileSource.resolve(&path, &DataType::String));
        assert_eq!(text, Ok(Value::String("sentinel".to_string())));

        let bytes = async_std::task::block_on(
            FileSource.resolve(&path, &DataType::Vec(Box::new(DataType::Byte))),
        );
        assert_eq!(
            bytes,
            Ok(Value::Packed(PackedArray::Byte(Arc::new(
                b"sentinel\n".to_vec()
            ))))
        );
    }

    #[cfg(feature = "environment")]
    #[test]
    fn environment_variables_are_given_as_text() {
        std::env::set_var("MELODIUM_SECRET_SOURCES_TEST", "sentinel");
        let text = async_std::task::block_on(
            EnvironmentSource.resolve("MELODIUM_SECRET_SOURCES_TEST", &DataType::String),
        );
        assert_eq!(text, Ok(Value::String("sentinel".to_string())));
        assert!(async_std::task::block_on(
            EnvironmentSource.resolve("MELODIUM_SECRET_SOURCES_UNSET", &DataType::String)
        )
        .is_err());
    }

    #[cfg(feature = "environment")]
    #[test]
    fn only_prefixed_environment_variables_are_given() {
        std::env::set_var("SECRET_SOURCES_TEST_UNPREFIXED", "sentinel");
        let refused = async_std::task::block_on(
            EnvironmentSource.resolve("SECRET_SOURCES_TEST_UNPREFIXED", &DataType::String),
        );
        assert_eq!(
            refused,
            Err("environment variable 'SECRET_SOURCES_TEST_UNPREFIXED' is not given to programs, only the ones starting with 'MELODIUM_SECRET_' are".to_string())
        );
    }

    #[test]
    fn environment_and_files_are_never_plainly_revealed() {
        #[cfg(feature = "environment")]
        assert!(!EnvironmentSource.plain_reveal());
        #[cfg(feature = "filesystem")]
        assert!(!FileSource.plain_reveal());
    }
}
