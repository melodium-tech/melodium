//! Module in charge of textual words parsing and analysis.
//!
//! This module contains low-level functions doing parsing and analysis of text, as well as [word elements](Word), the smallest unit of text that can be parsed.
//! All functions there are unicode-aware.

use core::fmt::{Display, Formatter};
use melodium_engine::designer::Reference;
use regex::Regex;
use std::str;
use std::sync::Arc;

/// Word, smallest unit of parsed text.
///
/// This structure embeds informations about a word, that can be anything like a name `MyFashionName`, value `12.345`, or any symbol like parenthesis, bracket, comma, etc.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Word {
    /// Literal text of the word.
    pub text: String,
    /// Kind of the word, may be None if the word is of an unknown kind.
    pub kind: Option<Kind>,
    /// Position of the word in the file.
    pub position: Position,
}

impl Default for Word {
    fn default() -> Self {
        Word {
            text: String::new(),
            kind: None,
            position: Position::default(),
        }
    }
}

/// Position of a word or element in text.
///
/// # Note
/// All positions (`absolute_position`, `line_position`) are expected to be bytes indexes, not chars.
#[derive(Default, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Position {
    /// Absolute position of the word inside the text script, as byte index.
    pub absolute_position: usize,
    /// Line where the word is (starting at 1).
    pub line_number: usize,
    /// Position of the word on its line , as byte index, zero meaning the first char after '\n'.
    pub line_position: usize,
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct PositionnedString {
    pub string: String,
    pub position: Position,
}

impl Reference for PositionnedString {}

impl PositionnedString {
    pub fn remove_indent(&mut self) {
        let mut prefix = None;
        for line in self.string.lines() {
            let trimmed_line = line.trim_start();
            if !trimmed_line.is_empty() {
                let whitespaces = line.split_at(line.find(trimmed_line).unwrap()).0;
                prefix = Some(whitespaces.to_string());
                break;
            }
        }

        if let Some(prefix) = prefix {
            let mut less_indented_string = String::new();
            for line in self.string.lines() {
                less_indented_string.push_str(line.strip_prefix(&prefix).unwrap_or(line));
                less_indented_string.push_str("\n");
            }
            self.string = less_indented_string;
        }
    }

    pub fn into_ref(&self) -> Arc<dyn Reference> {
        Arc::new(self.clone())
    }
}

impl From<&Word> for PositionnedString {
    fn from(word: &Word) -> Self {
        Self {
            string: word.text.clone(),
            position: word.position.clone(),
        }
    }
}

/// Kind of word.
///
/// "Kind" designates what the word fundamentaly is, meaning a `Name` is some text that designates name of something (including keyword), `Opening*` and `Closing*` are obvious, as well as `Equal`, `Colon`, `Comma`, etc.
///
/// Some "special" kinds of words, like `Comment`, `Annotations`, or `RightArrow` are there because they designates very specific patterns of text that can be easily and cheaply identified, and considered as single elements for all other parsing steps.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Kind {
    /// Comment, anything like `//…` or `/* … */`.
    Comment,
    /// Annotation, anything like `#…`.
    Annotation,
    /// `(`
    OpeningParenthesis,
    /// `)`
    ClosingParenthesis,
    /// `{`
    OpeningBrace,
    /// `}`
    ClosingBrace,
    /// `[`
    OpeningBracket,
    /// `]`
    ClosingBracket,
    /// `<`
    OpeningChevron,
    /// `>`
    ClosingChevron,
    /// `=`
    Equal,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `/`
    Slash,
    /// `_`
    Underscore,
    /// `+`,
    Plus,
    /// An arrow made of one or more `-` terminated by `>`, `--->`.
    RightArrow,
    /// Anything corresponding to a name, meaning anything that is composed of letters (Unicode definition) or numbers, but not starting with a number.
    Name,
    /// Same thing than `Name`, but having `@` in the first place.
    Context,
    /// Same thing than `Name`, but having `|` in the first place.
    Function,
    /// Anything matching a number, starting optionally with `-`, or any digit, and having an arbitrary number of digits, with at most one point `.` inside.
    Number,
    /// Any string starting and ending with `"` (with a preservation of `\"` and `\\`) or using incremental braces with `${` and `}`.
    String,
    /// A character enclosed by `'`
    Character,
    /// A byte composed of `0xFF`, `FF` being any hexadecimal value from range `[0-9A-F]`.
    Byte,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let str = match self {
            Kind::Comment => "// Comment",
            Kind::Annotation => "# Annotation",
            Kind::OpeningParenthesis => "(",
            Kind::ClosingParenthesis => ")",
            Kind::OpeningBrace => "{",
            Kind::ClosingBrace => "}",
            Kind::OpeningBracket => "[",
            Kind::ClosingBracket => "]",
            Kind::OpeningChevron => "<",
            Kind::ClosingChevron => ">",
            Kind::Equal => "=",
            Kind::Colon => ":",
            Kind::Comma => ",",
            Kind::Dot => ".",
            Kind::Slash => "/",
            Kind::Underscore => "_",
            Kind::Plus => "+",
            Kind::RightArrow => "->",
            Kind::Name => "name",
            Kind::Context => "context (@Context)",
            Kind::Function => "function (|function)",
            Kind::Number => "number",
            Kind::String => r#"string ("string")"#,
            Kind::Character => "character ('c')",
            Kind::Byte => "byte (0x2A)",
        };
        write!(f, "{}", str)
    }
}

/// Convenience structure for internal treatments.
///
/// Embeds different informations in fancy way, instead of a tuple.
#[derive(Debug)]
struct KindCheck {
    pub is_that_kind: bool,
    pub end_at: usize,
    pub is_well_formed: bool,
}

impl Default for KindCheck {
    fn default() -> Self {
        KindCheck {
            is_that_kind: false,
            end_at: 0,
            is_well_formed: false,
        }
    }
}

/// Make primary parsing of text, and return words inside it.
///
/// Returns a list of [words](Word) contained inside the text, as `Ok` if parsing went without error (implying every word has an associated kind), or as `Err` if something hasn't been recognized (the last word will be the erroneous one, and may be without kind).
///
/// See [expect_word] and [expect_word_kind] for example of usage.
pub fn get_words(script: &str) -> Result<Vec<Word>, Vec<Word>> {
    let mut words = Vec::new();
    let mut remaining_script = script.trim_start();
    let mut actual_position = script.len() - remaining_script.len();
    let mut kind_check: KindCheck;

    while !remaining_script.is_empty() {
        let kind: Option<Kind>;

        // Check if word is Comment.
        if {
            kind_check = manage_comment(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Comment);
        }
        // Check if word is Annotation
        else if {
            kind_check = manage_annotation(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Annotation);
        }
        // Check if word is OpeningParenthesis
        else if {
            kind_check = manage_single_char('(', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::OpeningParenthesis);
        }
        // Check if word is ClosingParenthesis
        else if {
            kind_check = manage_single_char(')', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::ClosingParenthesis);
        }
        // Check if word is OpeningBrace
        else if {
            kind_check = manage_single_char('{', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::OpeningBrace);
        }
        // Check if word is ClosingBrace
        else if {
            kind_check = manage_single_char('}', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::ClosingBrace);
        }
        // Check if word is OpeningBracket
        else if {
            kind_check = manage_single_char('[', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::OpeningBracket);
        }
        // Check if word is ClosingBracket
        else if {
            kind_check = manage_single_char(']', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::ClosingBracket);
        }
        // Check if word is OpeningChevron
        else if {
            kind_check = manage_single_char('<', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::OpeningChevron);
        }
        // Check if word is ClosingChevron
        else if {
            kind_check = manage_single_char('>', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::ClosingChevron);
        }
        // Check if word is Equal
        else if {
            kind_check = manage_single_char('=', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Equal);
        }
        // Check if word is Colon
        else if {
            kind_check = manage_single_char(':', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Colon);
        }
        // Check if word is Comma
        else if {
            kind_check = manage_single_char(',', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Comma);
        }
        // Check if word is Dot
        else if {
            kind_check = manage_single_char('.', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Dot);
        }
        // Check if word is Slash
        else if {
            kind_check = manage_single_char('/', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Slash);
        }
        // Check if word is Underscore
        else if {
            kind_check = manage_single_char('_', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Underscore);
        }
        // Check if word is Plus
        else if {
            kind_check = manage_single_char('+', remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Plus);
        }
        // Check if word is RightArrow
        else if {
            kind_check = manage_right_arrow(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::RightArrow);
        }
        // Check if word is Name
        else if {
            kind_check = manage_name(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Name);
        }
        // Check if word is Context
        else if {
            kind_check = manage_context(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Context);
        }
        // Check if word is Function
        else if {
            kind_check = manage_function(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Function);
        }
        // Check if word is Byte
        else if {
            kind_check = manage_byte(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Byte);
        }
        // Check if word is Number
        else if {
            kind_check = manage_number(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Number);
        }
        // Check if word is String
        else if {
            kind_check = manage_string(remaining_script);
            kind_check.is_that_kind
        } {
            eprintln!("String identified");
            kind = Some(Kind::String);
        }
        // Check if word is Char
        else if {
            kind_check = manage_char(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Character);
        }
        // The word is unknown
        else {
            kind_check = KindCheck {
                is_that_kind: false,
                end_at: 1,
                is_well_formed: false,
            };
            kind = None;
        }

        eprintln!("Kind check: {kind_check:?}");

        if let Some(splitted_script) = remaining_script.split_at_checked(kind_check.end_at) {
            let (line, pos_in_line) = get_line_pos(script, actual_position);
            let word = Word {
                text: splitted_script.0.to_string(),
                position: Position {
                    absolute_position: actual_position,
                    line_position: pos_in_line,
                    line_number: line,
                },
                kind: kind,
            };

            words.push(word);

            if !kind_check.is_well_formed {
                return Err(words);
            } else {
                let after_word = splitted_script.1.trim_start();
                actual_position += remaining_script.len() - after_word.len();
                remaining_script = after_word;
            }
        } else {
            return Err(words);
        }
    }

    Ok(words)
}

fn get_line_pos(text: &str, pos: usize) -> (usize, usize) {
    let considered_text = text.split_at(pos).0;
    let newlines_indices = considered_text.match_indices('\n');

    let counter = newlines_indices.clone();
    let lines = counter.count() + 1;

    let line_start;
    if lines > 1 {
        line_start = newlines_indices.last().unwrap().0 + 1;
    } else {
        line_start = 0;
    }

    let pos_in_line = pos - line_start;

    (lines, pos_in_line)
}

fn manage_comment(text: &str) -> KindCheck {
    if text.starts_with("//") {
        let end_of_comment = text.find('\n');
        KindCheck {
            is_that_kind: true,
            end_at: end_of_comment.unwrap_or_else(|| text.len()),
            is_well_formed: true,
        }
    } else if text.starts_with("/*") {
        let end_of_comment = text.find("*/");
        KindCheck {
            is_that_kind: true,
            end_at: end_of_comment.unwrap_or_else(|| text.len()) + 2,
            is_well_formed: end_of_comment.is_some(),
        }
    } else {
        KindCheck::default()
    }
}

fn manage_annotation(text: &str) -> KindCheck {
    if text.starts_with('#') {
        let end_of_annotation = text.find('\n');
        KindCheck {
            is_that_kind: true,
            end_at: end_of_annotation.unwrap_or_else(|| text.len()),
            is_well_formed: true,
        }
    } else {
        KindCheck::default()
    }
}

fn manage_single_char(c: char, text: &str) -> KindCheck {
    if text.starts_with(c) {
        KindCheck {
            is_that_kind: true,
            end_at: 1,
            is_well_formed: true,
        }
    } else {
        KindCheck::default()
    }
}

fn manage_right_arrow(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_RIGHT_ARROW: Regex = Regex::new(r"^-+>").unwrap();
    }
    let mat = REGEX_RIGHT_ARROW.find(text);
    if mat.is_some() {
        KindCheck {
            is_that_kind: true,
            end_at: mat.unwrap().end(),
            is_well_formed: true,
        }
    } else {
        KindCheck::default()
    }
}

fn manage_name(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_NAME: Regex =
            Regex::new(r"^[\p{Alphabetic}\p{M}\p{Pc}\p{Join_Control}]\w*").unwrap();
    }
    let mat = REGEX_NAME.find(text);
    if mat.is_some() {
        KindCheck {
            is_that_kind: true,
            end_at: mat.unwrap().end(),
            is_well_formed: true,
        }
    } else {
        KindCheck::default()
    }
}

fn manage_context(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_CONTEXT: Regex =
            Regex::new(r"^@[\p{Alphabetic}\p{M}\p{Pc}\p{Join_Control}]\w*").unwrap();
    }
    let mat = REGEX_CONTEXT.find(text);
    if mat.is_some() {
        KindCheck {
            is_that_kind: true,
            end_at: mat.unwrap().end(),
            is_well_formed: true,
        }
    } else {
        KindCheck::default()
    }
}

fn manage_function(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_CONTEXT: Regex =
            Regex::new(r"^\|[\p{Alphabetic}\p{M}\p{Pc}\p{Join_Control}]\w*").unwrap();
    }
    let mat = REGEX_CONTEXT.find(text);
    if mat.is_some() {
        KindCheck {
            is_that_kind: true,
            end_at: mat.unwrap().end(),
            is_well_formed: true,
        }
    } else {
        KindCheck::default()
    }
}

fn manage_number(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_NUMBER: Regex = Regex::new(r"^-?[0-9]*\.?[0-9]+").unwrap();
    }
    let mat = REGEX_NUMBER.find(text);
    if mat.is_some() {
        KindCheck {
            is_that_kind: true,
            end_at: mat.unwrap().end(),
            is_well_formed: true,
        }
    } else {
        KindCheck::default()
    }
}

fn manage_string(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_STRING: Regex = Regex::new(r##"^"(?:[^"\\]|\\.)*""##).unwrap();
    }
    if text.starts_with('"') {
        let mat = REGEX_STRING.find(text);
        if mat.is_some() {
            KindCheck {
                is_that_kind: true,
                end_at: mat.unwrap().end(),
                is_well_formed: true,
            }
        } else {
            KindCheck {
                is_that_kind: true,
                end_at: text.len(),
                is_well_formed: false,
            }
        }
    } else if text.starts_with("${") {
        let num_braces = text.chars().skip(1).take_while(|c| *c == '{').count();
        let mut end_braces: String = "}".into();
        for _ in 1..num_braces {
            end_braces.push('}');
        }
        eprintln!("num_braces: {num_braces}, end_braces: {end_braces}");
        if let Some(end_string_position) = text.find(&end_braces) {
            KindCheck {
                is_that_kind: true,
                end_at: end_string_position + num_braces,
                is_well_formed: false,
            }
        } else {
            KindCheck {
                is_that_kind: true,
                end_at: text.len(),
                is_well_formed: false,
            }
        }
    } else {
        KindCheck::default()
    }
}

fn manage_char(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_CHAR: Regex = Regex::new(r##"^'(?:[^'\]|\.)+'"##).unwrap();
    }
    if text.starts_with('\'') {
        let mat = REGEX_CHAR.find(text);
        if mat.is_some() {
            KindCheck {
                is_that_kind: true,
                end_at: mat.unwrap().end(),
                is_well_formed: true,
            }
        } else {
            KindCheck {
                is_that_kind: true,
                end_at: text.len(),
                is_well_formed: false,
            }
        }
    } else {
        KindCheck::default()
    }
}

fn manage_byte(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_BYTE: Regex = Regex::new(r##"^(?:0x[0-9A-F]{2})"##).unwrap();
    }
    if text.starts_with("0x") {
        let mat = REGEX_BYTE.find(text);
        if mat.is_some() {
            KindCheck {
                is_that_kind: true,
                end_at: mat.unwrap().end(),
                is_well_formed: true,
            }
        } else {
            KindCheck {
                is_that_kind: true,
                end_at: text.len(),
                is_well_formed: false,
            }
        }
    } else {
        KindCheck::default()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_well_formated_comments() {
        let comments = "// A comment
        //Anoter comment
        Not_a_comment
        /*A continuous comment*/
        /* A
         * quite
         * long
         * comment
         */
        /* A shorter comment */";

        let words = get_words(comments).unwrap();
        let kinds: Vec<bool> = words
            .iter()
            .map(|w| w.kind == Some(Kind::Comment))
            .collect();

        assert_eq!(vec![true, true, false, true, true, true], kinds);
    }

    #[test]
    fn test_well_formated_numbers() {
        let numbers = "0
        -12
        1.234
        Not_a_number
        -1.234
        -0
        00000000000000000000000000000";

        let words = get_words(numbers).unwrap();
        let kinds: Vec<bool> = words.iter().map(|w| w.kind == Some(Kind::Number)).collect();

        assert_eq!(vec![true, true, true, false, true, true, true], kinds);
    }
}
