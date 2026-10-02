use async_trait::async_trait;
use melodium_common::descriptor::DataType;
use melodium_common::executive::{PackedArray, SecretSource, Value};
use std::sync::Arc;

/// Gives the value of a text source as `datatype`, if it is `string` or `Vec<byte>`.
fn text_value(text: String, datatype: &DataType, origin: &str) -> Result<Value, String> {
    match datatype {
        DataType::String => Ok(Value::String(text)),
        DataType::Vec(inner) if matches!(**inner, DataType::Byte) => Ok(Value::Packed(
            PackedArray::Byte(Arc::new(text.into_bytes())),
        )),
        other => Err(format!("{origin} cannot be given as {other}")),
    }
}

/// `env:NAME` source, giving the value of an environment variable.
#[derive(Debug)]
pub struct EnvironmentSource;

#[async_trait]
impl SecretSource for EnvironmentSource {
    async fn resolve(&self, path: &str, datatype: &DataType) -> Result<Value, String> {
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
#[derive(Debug)]
pub struct FileSource;

#[async_trait]
impl SecretSource for FileSource {
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
