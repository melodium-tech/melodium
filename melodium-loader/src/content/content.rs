
use core::str::Utf8Error;
use super::script::{Script, ScriptError};

pub enum Content {
    Script(Script),
}

impl Content {
    pub fn new(path: &str, content: &[u8]) -> Result<Self, ContentError> {
        // Currently only script content is supported
        let path = path.to_string();
        let text = std::str::from_utf8(content).map_err(|error| ContentError::Utf8Error { path: path.clone(), error })?;

        let content = Script::new(path.clone(), text).map_err(|errors| ContentError::ScriptErrors { path, errors })?;

        Ok(Self::Script(content))
    }
}
pub enum ContentError {
    Utf8Error { path: String, error: Utf8Error },
    ScriptErrors { path: String, errors: Vec<ScriptError> },
}
