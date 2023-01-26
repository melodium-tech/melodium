// Depending on activated features, some parts may or may not be used or reached.
#![allow(unused, unreachable_patterns)]

#[cfg(feature = "script")]
use super::script::{Script, ScriptBuildLevel, ScriptError};
use core::str::Utf8Error;
use melodium_common::descriptor::{Collection, Identifier};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct Content {
    content: ContentType,
    descriptors_building: Mutex<()>,
}

impl Content {
    pub fn new(path: &str, content: &[u8]) -> Result<Self, ContentError> {
        // Currently only script content is supported
        #[cfg(feature = "script")]
        {
            let path = path.to_string();
            let text = std::str::from_utf8(content).map_err(|error| ContentError::Utf8Error {
                path: path.clone(),
                error,
            })?;

            let content = Script::new(path.clone(), text)
                .map_err(|errors| ContentError::ScriptErrors { path, errors })?;

            Ok(Self {
                content: ContentType::Script(content),
                descriptors_building: Mutex::new(()),
            })
        }
        #[cfg(not(feature = "script"))]
        Err(ContentError::UnsupportedContent)
    }

    #[allow(unused)]
    pub fn match_identifier(&self, identifier: &Identifier) -> bool {
        match &self.content {
            #[cfg(feature = "script")]
            ContentType::Script(script) => script.match_identifier(identifier),
            _ => false,
        }
    }

    pub fn level(&self) -> ContentLevel {
        match &self.content {
            #[cfg(feature = "script")]
            ContentType::Script(script) => match script.build_level() {
                ScriptBuildLevel::None => ContentLevel::Exists,
                ScriptBuildLevel::DescriptorsMade => ContentLevel::Described,
                ScriptBuildLevel::DesignMade => ContentLevel::Designed,
            },
            _ => ContentLevel::Exists,
        }
    }

    pub fn require(&self) -> Vec<Identifier> {
        match &self.content {
            #[cfg(feature = "script")]
            ContentType::Script(script) => script.need(),
            _ => Vec::new(),
        }
    }

    pub fn provide(&self) -> Vec<Identifier> {
        match &self.content {
            #[cfg(feature = "script")]
            ContentType::Script(script) => script.provide(),
            _ => Vec::new(),
        }
    }

    pub fn try_lock(&self) -> Result<MutexGuard<()>, ()> {
        match self.descriptors_building.try_lock() {
            Ok(guard) => Ok(guard),
            Err(_) => Err(()),
        }
    }

    pub fn insert_descriptors(&self, collection: &mut Collection) -> Result<(), ContentError> {
        match &self.content {
            #[cfg(feature = "script")]
            ContentType::Script(script) => {
                script
                    .make_descriptors(collection)
                    .map_err(|e| ContentError::ScriptErrors {
                        path: script.path().to_string(),
                        errors: e,
                    })?
            }
            _ => {}
        }
        Ok(())
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> Result<(), ContentError> {
        match &self.content {
            #[cfg(feature = "script")]
            ContentType::Script(script) => {
                script
                    .make_design(collection)
                    .map_err(|e| ContentError::ScriptErrors {
                        path: script.path().to_string(),
                        errors: e,
                    })?
            }
            _ => {}
        }
        Ok(())
    }
}

enum ContentType {
    #[cfg(feature = "script")]
    Script(Script),
}

#[derive(Clone)]
pub enum ContentError {
    UnsupportedContent,
    Utf8Error {
        path: String,
        error: Utf8Error,
    },
    #[cfg(feature = "script")]
    ScriptErrors {
        path: String,
        errors: Vec<ScriptError>,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum ContentLevel {
    Exists,
    Described,
    Designed,
}
