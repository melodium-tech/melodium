// Depending on activated features, some parts may or may not be used or reached.
#![allow(unused, unreachable_patterns)]

#[cfg(feature = "script")]
use super::script::{Script, ScriptBuildLevel};
use core::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use melodium_common::descriptor::{
    Collection, ContentError as CommonContentError, Identifier, Status, Version,
};
#[cfg(feature = "script")]
use melodium_lang::{error::ScriptErrors, ScriptError};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug)]
pub struct Content {
    content: ContentType,
    descriptors_building: Mutex<()>,
}

impl Content {
    pub fn new(path: &str, content: &[u8]) -> ContentResult<Self> {
        // Currently only script content is supported
        #[cfg(feature = "script")]
        {
            let text = match std::str::from_utf8(content).map_err(|error| {
                ContentResult::new_failure(ContentError::Utf8Error {
                    path: path.to_string(),
                    error,
                })
            }) {
                Ok(text) => text,
                Err(err) => return err,
            };

            Script::new(&path, text)
                .convert_failure_errors(|error| ContentError::ScriptError {
                    path: path.to_string(),
                    error,
                })
                .and_then(|content| {
                    ContentResult::new_success(Self {
                        content: ContentType::Script(content),
                        descriptors_building: Mutex::new(()),
                    })
                })
        }
        #[cfg(not(feature = "script"))]
        ContentResult::new_failure(ContentError::UnsupportedContent)
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

    pub fn insert_descriptors(&self, version: &Version, collection: &mut Collection) -> ContentResult<()> {
        match &self.content {
            #[cfg(feature = "script")]
            ContentType::Script(script) => script
                .make_descriptors(version, collection)
                .convert_failure_errors(|error| ContentError::ScriptError {
                    path: script.path().to_string(),
                    error,
                }),
            _ => ContentResult::new_success(()),
        }
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> ContentResult<()> {
        match &self.content {
            #[cfg(feature = "script")]
            ContentType::Script(script) => {
                script
                    .make_design(collection)
                    .convert_failure_errors(|error| ContentError::ScriptError {
                        path: script.path().to_string(),
                        error,
                    })
            }
            _ => ContentResult::new_success(()),
        }
    }
}

#[derive(Debug)]
enum ContentType {
    #[cfg(feature = "script")]
    Script(Script),
}

#[derive(Clone, Debug)]
pub enum ContentError {
    UnsupportedContent,
    Utf8Error {
        path: String,
        error: Utf8Error,
    },
    #[cfg(feature = "script")]
    ScriptError {
        path: String,
        error: ScriptError,
    },
}

impl Display for ContentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentError::UnsupportedContent => write!(f, "Content is not supported"),
            ContentError::Utf8Error { path, error } => {
                write!(f, "Encoding error '{error}' on {path}")
            }
            #[cfg(feature = "script")]
            ContentError::ScriptError { path, error } => {
                write!(f, "Script error on {path}: {error}")
            }
        }
    }
}

impl CommonContentError for ContentError {}

pub type ContentResult<T> = Status<T, ContentError, ContentError>;

#[derive(Clone, Copy, Debug)]
pub enum ContentLevel {
    Exists,
    Described,
    Designed,
}
