//! Provides Mélodium script error management.
//!
//! The main type of this module is [ScriptError](./struct.ScriptError.html), which handles most of the management, combined with kind of errors detailed with [ScriptErrorKind](./enum.ScriptErrorKind.html).

use super::text::Position;
use melodium_engine::LogicError;
use std::convert;
use std::error;
use std::fmt;

/// Handles and describe a Mélodium script error.
///
/// Most of the properties are deeply related with [Word](../text/word/struct.Word.html).
///
/// # Note
/// All positions (`absolute_position`, `line_position`) are expected to be bytes indexes, not chars.
#[derive(Debug, Clone)]
pub struct ScriptError {
    /// Message associated with the error.
    pub message: String,
    /// Literal text of the word.
    pub word: String,
    /// Kind of error.
    pub kind: ScriptErrorKind,
    /// Position of the erroneous element.
    pub position: Position,
}

/// Kind of script error that might happens.
#[derive(Debug, Clone)]
pub enum ScriptErrorKind {
    /// The error is related to a specific word that disable script to work.
    Word,
    /// The error is about an unexcpected end of script.
    EndOfScript,
    /// The error is about semantic.
    Semantic,
    /// The error is about file.
    File,
    /// The error comes from logic.
    Logic(LogicError),
    /// No descriptor associated
    NoDescriptor,
}

impl ScriptError {
    /// Creates a new error of Word kind.
    ///
    /// The ScriptError created that way will be of [ScriptErrorKind::Word](./enum.ScriptErrorKind.html#variant.Word) kind.
    /// Each parameter matches the properties of ScriptError.
    pub fn word(message: String, word: String, position: Position) -> Self {
        Self {
            message,
            word,
            position,
            kind: ScriptErrorKind::Word,
        }
    }

    /// Creates a new error of EndOfScript kind.
    ///
    /// The ScriptError created that way will be of [ScriptErrorKind::EndOfScript](./enum.ScriptErrorKind.html#variant.EndOfScript) kind.
    pub fn end_of_script(message: String) -> Self {
        Self {
            message,
            word: String::new(),
            position: Position {
                line_number: 0,
                line_position: 0,
                absolute_position: 0,
            },
            kind: ScriptErrorKind::EndOfScript,
        }
    }

    pub fn semantic(message: String, position: Position) -> Self {
        Self {
            message,
            word: String::new(),
            position,
            kind: ScriptErrorKind::Semantic,
        }
    }

    pub fn file(message: String) -> Self {
        Self {
            message,
            word: String::new(),
            position: Position {
                line_number: 0,
                line_position: 0,
                absolute_position: 0,
            },
            kind: ScriptErrorKind::File,
        }
    }

    pub fn logic(logic_error: LogicError, position: Position) -> Self {
        Self {
            message: String::new(),
            word: String::new(),
            position,
            kind: ScriptErrorKind::Logic(logic_error),
        }
    }

    pub fn no_descriptor() -> Self {
        Self {
            message: String::new(),
            word: String::new(),
            position: Position {
                line_number: 0,
                line_position: 0,
                absolute_position: 0,
            },
            kind: ScriptErrorKind::NoDescriptor,
        }
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ScriptErrorKind::Word => {
                if self.word.len() > 0 && self.word.len() <= 12 {
                    write!(
                        f,
                        "\"{}\" at line {} position {} (absolute {}): {}",
                        self.word,
                        self.position.line_number,
                        self.position.line_position,
                        self.position.absolute_position,
                        self.message
                    )
                } else {
                    write!(
                        f,
                        "line {} position {} (absolute {}): {}",
                        self.position.line_number,
                        self.position.line_position,
                        self.position.absolute_position,
                        self.message
                    )
                }
            }
            ScriptErrorKind::EndOfScript => write!(f, "{}", self.message),
            ScriptErrorKind::Semantic => write!(
                f,
                "line {} position {} (absolute {}): {}",
                self.position.line_number,
                self.position.line_position,
                self.position.absolute_position,
                self.message
            ),
            ScriptErrorKind::File => write!(f, "{}", self.message),
            ScriptErrorKind::Logic(le) => write!(
                f,
                "line {} position {} (absolute {}): {}",
                self.position.line_number,
                self.position.line_position,
                self.position.absolute_position,
                le
            ),
            ScriptErrorKind::NoDescriptor => write!(
                f,
                "no descriptor ready, this is an internal error to report to development team"
            ),
        }
    }
}

impl convert::From<LogicError> for ScriptError {
    fn from(le: LogicError) -> Self {
        ScriptError::logic(
            le,
            Position {
                line_number: 0,
                line_position: 0,
                absolute_position: 0,
            },
        )
    }
}

impl error::Error for ScriptError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

macro_rules! wrap_logic_error {
    ($possible_error:expr, $position:expr) => {
        match $possible_error {
            Err(le) => return Err(ScriptError::logic(le, $position)),
            Ok(v) => v,
        }
    };
}
pub(crate) use wrap_logic_error;
