
use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ScriptError {
    pub absolute_position: usize,
    pub line: usize,
    pub line_position: usize,
    pub message: String,
    pub word: String,
    pub kind: ScriptErrorKind,
}

#[derive(Debug, Copy, Clone)]
pub enum ScriptErrorKind {
    Word,
    EndOfScript,
}

impl ScriptError {
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
