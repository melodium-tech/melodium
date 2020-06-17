
//! Provides Mélodium script error management.
//! 
//! The main type of this module is [ScriptError](./struct.ScriptError.html), which handles most of the management, combined with kind of errors detailed with [ScriptErrorKind](./enum.ScriptErrorKind.html).

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
    /// Absolute position of the word inside the text script, as byte index.
    pub absolute_position: usize,
    /// Line where the word is (starting at 1).
    pub line: usize,
    /// Position of the word on its line , as byte index, zero meaning the first char after '\n'.
    pub line_position: usize,
    /// Message associated with the error.
    pub message: String,
    /// Literal text of the word.
    pub word: String,
    /// Kind of error.
    pub kind: ScriptErrorKind,
}

/// Kind of script error that might happens.
#[derive(Debug, Copy, Clone)]
pub enum ScriptErrorKind {
    /// The error is related to a specific word that disable script to work.
    Word,
    /// The error is about an unexcpected end of script.
    EndOfScript,
    Semantic,
}

impl ScriptError {
    /// Creates a new error.
    /// 
    /// The ScriptError created that way will be of [ScriptErrorKind::Word](./enum.ScriptErrorKind.html#variant.Word) kind.
    /// Each parameter matches the properties of ScriptError.
    pub fn new(message: String, word: String, line: usize, line_position: usize, absolute_position: usize) -> Self {
        Self {
            message,
            word,
            line,
            line_position,
            absolute_position,
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
            line: 0,
            line_position: 0,
            absolute_position: 0,
            kind: ScriptErrorKind::EndOfScript,
        }
    }

    pub fn semantic(message: String) -> Self {
        Self {
            message,
            word: String::new(),
            line: 0,
            line_position: 0,
            absolute_position: 0,
            kind: ScriptErrorKind::Semantic,
        }
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ScriptErrorKind::Word =>
            if self.word.len() > 0 && self.word.len() <= 12 {
                write!(f, "\"{}\" at line {} position {} (absolute {}): {}", self.word, self.line, self.line_position, self.absolute_position, self.message)
            }
            else {
                write!(f, "line {} position {} (absolute {}): {}", self.line, self.line_position, self.absolute_position, self.message)
            },
            ScriptErrorKind::EndOfScript => write!(f, "{}", self.message),
        }
        
    }
}

impl error::Error for ScriptError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
