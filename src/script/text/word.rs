
//! Module in charge of textual words parsing and analysis.
//! 
//! This module contains low-level functions doing parsing and analysis of text, as well as [word elements](struct.Word.html), the smallest unit of text that can be parsed.
//! All functions there are unicode-aware.

extern crate regex;

use std::str;
use regex::Regex;
use crate::script::error::ScriptError;

/// Word, smallest unit of parsed text.
/// 
/// This structure embeds informations about a word, that can be anything like a name `MyFashionName`, value `12.345`, or any symbol like parenthesis, bracket, comma, etc.
#[derive(Debug, Clone)]
pub struct Word {
    /// Literal text of the word.
    pub text: String,
    /// Kind of the word, may be None if the word is of an unknown kind.
    pub kind: Option<Kind>,
    /// Position of the word in the file.
    pub position: Position,
}

/// Position of a word or element in text.
/// 
/// # Note
/// All positions (`absolute_position`, `line_position`) are expected to be bytes indexes, not chars.
#[derive(Default, Debug, Copy, Clone)]
pub struct Position {
    /// Absolute position of the word inside the text script, as byte index.
    pub absolute_position: usize,
    /// Line where the word is (starting at 1).
    pub line_number: usize,
    /// Position of the word on its line , as byte index, zero meaning the first char after '\n'.
    pub line_position: usize,
}

#[derive(Default, Debug, Clone)]
pub struct PositionnedString {
    pub string: String,
    pub position: Position,
}

/// Kind of word.
/// 
/// "Kind" designates what the word fundamentaly is, meaning a `Name` is some text that designates name of something (including keyword), `Opening*` and `Closing*` are obvious, as well as `Equal`, `Colon`, `Comma`, etc.
/// 
/// Some "special" kinds of words, like `Comment`, `Annotations`, or `RightArrow` are there because they designates very specific patterns of text that can be easily and cheaply identified, and considered as single elements for all other parsing steps.
#[derive(Debug, PartialEq, Copy, Clone)]
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
    /// An arrow made of one or more `-` terminated by `>`, `--->`.
    RightArrow,
    /// Anything corresponding to a name, meaning anything that is composed of letters (Unicode definition) or numbers, but not starting with a number.
    Name,
    /// Same thing than `Name`, but having `@` in the first place.
    Reference,
    /// Anything matching a number, starting with `+`, `-`, or any digit, and having an arbitrary number of digits, with at most one point `.` inside.
    Number,
    /// Any string starting and ending with `"` (with a preservation of `\"` and `\\`).
    String,
}

/// Convenience structure for internal treatments.
/// 
/// Embeds different informations in fancy way, instead of a tuple.
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

/// Give next word or create error.
/// 
/// Return the next word if any, or create a [ScriptError](../../error/struct.ScriptError.html), with `error_str` as message.
/// This function always increment `iter` from one.
/// 
/// ```
/// # use melodium_rust::script::text::word::*;
/// # use melodium_rust::script::error::ScriptError;
/// let words = get_words("myNumber= 876").unwrap();
/// let mut iter = words.iter();
/// 
/// let name = expect_word("Word expected.", &mut iter)?;
/// let equal = expect_word("Word expected.", &mut iter)?;
/// let value = expect_word("Word expected.", &mut iter)?;
/// 
/// assert_eq!(name.kind, Some(Kind::Name));
/// assert_eq!(equal.kind, Some(Kind::Equal));
/// assert_eq!(value.kind, Some(Kind::Number));
/// # Ok::<(), ScriptError>(())
/// ```
pub fn expect_word(error_str: &'static str, iter: &mut std::slice::Iter<Word>) -> Result<Word, ScriptError> {
    let word = iter.next();
    if word.is_some() {
        let word = word.unwrap();
        if word.kind.is_some() {
            return Ok(word.clone());
        }
    }

    Err(ScriptError::end_of_script(error_str.to_string()))
}

/// Check aext word kind and returns its text, or create error.
/// 
/// Return next word text if any and matches `kind`, else create a [ScriptError](../../error/struct.ScriptError.html), with `error_str` as message.
/// This function always increment `iter` from one.
/// 
/// ```
/// # use melodium_rust::script::text::word::*;
/// # use melodium_rust::script::error::ScriptError;
/// let words = get_words("myNumber= 876").unwrap();
/// let mut iter = words.iter();
/// 
/// let name = expect_word_kind(Kind::Name, "Name expected.", &mut iter)?;
/// let equal = expect_word_kind(Kind::Equal, "Equal sign expected.", &mut iter)?;
/// let value = expect_word_kind(Kind::Number, "Number expected.", &mut iter)?;
/// 
/// assert_eq!(name.string, "myNumber");
/// assert_eq!(equal.string, "=");
/// assert_eq!(value.string, "876");
/// # Ok::<(), ScriptError>(())
/// ```
pub fn expect_word_kind(kind: Kind, error_str: &'static str, iter: &mut std::slice::Iter<Word>) -> Result<PositionnedString, ScriptError> {
    let word = iter.next();
    if word.is_some() {
        let word = word.unwrap();
        if word.kind == Some(kind) {
            Ok(PositionnedString{string: word.text.to_string(), position: word.position})
        }
        else {
            Err(ScriptError::word(error_str.to_string(), word.text.to_string(), word.position))
        }
    }
    else {
        Err(ScriptError::end_of_script(error_str.to_string()))
    }
}

/// Make primary parsing of text, and return words inside it.
/// 
/// Returns a list of [words](./struct.Word.html) contained inside the text, as `Ok` if parsing went without error (implying every word has an associated kind), or as `Err` if something hasn't been recognized (the last word will be the erroneous one, and may be without kind).
/// 
/// See [expect_word](./fn.expect_word.html) and [expect_word_kind](./fn.expect_word_kind.html) for example of usage.
pub fn get_words(script: & str) -> Result<Vec<Word>, Vec<Word>> {
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
        // Check if word is Reference
        else if {
            kind_check = manage_reference(remaining_script);
            kind_check.is_that_kind
        } {
            kind = Some(Kind::Reference);
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
            kind = Some(Kind::String);
        }
        // The word is unkown
        else {
            kind_check = KindCheck {
                is_that_kind: false,
                end_at: 1,
                is_well_formed: false,
            };
            kind = None;
        }
        
        let splitted_script = remaining_script.split_at(kind_check.end_at);
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
            return Err(words)
        }
        else {
            let after_word = splitted_script.1.trim_start();
            actual_position += remaining_script.len() - after_word.len();
            remaining_script = after_word;
        }

    }

    Ok(words)
}

fn get_line_pos(text: & str, pos: usize) -> (usize, usize) {
    let considered_text = text.split_at(pos).0;
    let newlines_indices = considered_text.match_indices('\n');

    let counter = newlines_indices.clone();
    let lines = counter.count() + 1;

    let line_start;
    if lines > 1 {
        line_start = newlines_indices.last().unwrap().0 + 1;
    }
    else {
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
            is_well_formed: true
        }
    }
    else if text.starts_with("/*") {
        let end_of_comment = text.find("*/");
        KindCheck {
            is_that_kind: true,
            end_at: end_of_comment.unwrap_or_else(|| text.len()) + 2,
            is_well_formed: end_of_comment.is_some()
        }
    }
    else { KindCheck::default() }
}

fn manage_annotation(text: &str) -> KindCheck {
    if text.starts_with('#') {
        let end_of_annotation = text.find('\n');
        KindCheck {
            is_that_kind: true,
            end_at: end_of_annotation.unwrap_or_else(|| text.len()),
            is_well_formed: true,
        }
    }
    else { KindCheck::default() }
}

fn manage_single_char(c: char, text: &str) -> KindCheck {
    if text.starts_with(c) {
        KindCheck {
            is_that_kind: true,
            end_at: 1,
            is_well_formed: true,
        }
    }
    else { KindCheck::default() }
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
    }
    else { KindCheck::default() }
}

fn manage_name(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_NAME: Regex = Regex::new(r"^[\p{Alphabetic}\p{M}\p{Pc}\p{Join_Control}]\w*").unwrap();
    }
    let mat = REGEX_NAME.find(text);
    if mat.is_some() {
        KindCheck {
            is_that_kind: true,
            end_at: mat.unwrap().end(),
            is_well_formed: true,
        }
    }
    else { KindCheck::default() }
}

fn manage_reference(text: &str) -> KindCheck {
    lazy_static! {
        static ref REGEX_REFERENCE: Regex = Regex::new(r"^@[\p{Alphabetic}\p{M}\p{Pc}\p{Join_Control}]\w*").unwrap();
    }
    let mat = REGEX_REFERENCE.find(text);
    if mat.is_some() {
        KindCheck {
            is_that_kind: true,
            end_at: mat.unwrap().end(),
            is_well_formed: true,
        }
    }
    else { KindCheck::default() }
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
    }
    else { KindCheck::default() }
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
        }
        else {
            KindCheck{
                is_that_kind: true,
                end_at: text.len(),
                is_well_formed: false,
            }
        }
    }
    else { KindCheck::default() }
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
        let kinds : Vec<bool> = words.iter().map(|w| w.kind == Some(Kind::Comment)).collect();

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
        let kinds : Vec<bool> = words.iter().map(|w| w.kind == Some(Kind::Number)).collect();

        assert_eq!(vec![true, true, true, false, true, true, true], kinds);
    }
}

