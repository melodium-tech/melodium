
//! Provides Mélodium script error management.
//! 
//! The main type of this module is [ScriptError](./struct.ScriptError.html), which handles most of the management, combined with kind of errors detailed with [ScriptErrorKind](./enum.ScriptErrorKind.html).

use std::error;
use std::fmt;
use super::text::Position;

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
#[derive(Debug, Copy, Clone)]
pub enum ScriptErrorKind {
    /// The error is related to a specific word that disable script to work.
    Word,
    /// The error is about an unexcpected end of script.
    EndOfScript,
    /// The error is about semantic.
    Semantic,
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
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ScriptErrorKind::Word =>
            if self.word.len() > 0 && self.word.len() <= 12 {
                write!(f, "\"{}\" at line {} position {} (absolute {}): {}", self.word, self.position.line_number, self.position.line_position, self.position.absolute_position, self.message)
            }
            else {
                write!(f, "line {} position {} (absolute {}): {}", self.position.line_number, self.position.line_position, self.position.absolute_position, self.message)
            },
            ScriptErrorKind::EndOfScript => write!(f, "{}", self.message),
            ScriptErrorKind::Semantic => write!(f, "{}", self.message),
        }
        
    }
}

impl error::Error for ScriptError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
