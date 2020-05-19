

extern crate regex;

use std::str;
use regex::Regex;
use crate::script::error::ScriptError;

#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,
    pub absolute_position: usize,
    pub line: usize,
    pub line_position: usize,
    pub kind: Option<Kind>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Kind {
    Comment,
    Annotation,
    OpeningParenthesis,
    ClosingParenthesis,
    OpeningBrace,
    ClosingBrace,
    OpeningBracket,
    ClosingBracket,
    OpeningChevron,
    ClosingChevron,
    Equal,
    Colon,
    Comma,
    Dot,
    Slash,
    RightArrow,
    Name,
    Reference,
    Number,
    String,
}

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

pub fn expect_word_kind(kind: Kind, error_str: &'static str, iter: &mut std::slice::Iter<Word>) -> Result<String, ScriptError> {
    let word = iter.next();
    if word.is_some() {
        let word = word.unwrap();
        if word.kind == Some(kind) {
            Ok(word.text.to_string())
        }
        else {
            Err(ScriptError::new(error_str.to_string(), word.text.to_string(), word.line, word.line_position, word.absolute_position))
        }
    }
    else {
        Err(ScriptError::end_of_script(error_str.to_string()))
    }
}

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
            absolute_position: actual_position,
            line: line,
            line_position: pos_in_line,
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

