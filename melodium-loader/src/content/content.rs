
use core::str::Utf8Error;
use super::script::{Script, ScriptError, ScriptBuildLevel};
use melodium_common::descriptor::{Collection, Identifier};
use std::sync::Arc;

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

    pub fn match_identifier(&self, identifier: &Identifier) -> bool {
        match self {
            Self::Script(script) => script.match_identifier(identifier),
            _ => false,
        }
    }

    pub fn level(&self) -> ContentLevel {
        match self {
            Self::Script(script) => 
                match script.build_level() {
                    ScriptBuildLevel::None => ContentLevel::Exists,
                    ScriptBuildLevel::DescriptorsMade => ContentLevel::Described,
                    ScriptBuildLevel::DesignMade => ContentLevel::Designed,
                },
            _ => ContentLevel::Exists,
        }
    }

    pub fn require(&self) -> Vec<Identifier> {
        match self {
            Self::Script(script) => script.need(),
            _ => Vec::new(),
        }
    }

    pub fn provide(&self) -> Vec<Identifier> {
        match self {
            Self::Script(script) => script.provide(),
            _ => Vec::new(),
        }
    }

    pub fn insert_descriptors(&self, collection: &mut Collection) -> Result<(), ContentError> {
        match self {
            Self::Script(script) => script.make_descriptors(collection).map_err(|e| ContentError::ScriptErrors { path: script.path().to_string(), errors: e })?,
            _ => {},
        }
        Ok(())
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> Result<(), ContentError> {
        match self {
            Self::Script(script) => script.make_design(collection).map_err(|e| ContentError::ScriptErrors { path: script.path().to_string(), errors: e })?,
            _ => {},
        }
        Ok(())
    }
}
pub enum ContentError {
    Utf8Error { path: String, error: Utf8Error },
    ScriptErrors { path: String, errors: Vec<ScriptError> },
}

#[derive(Clone, Copy, Debug)]
pub enum ContentLevel {
    Exists,
    Described,
    Designed,
}
