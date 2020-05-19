
use crate::script::error::ScriptError;

use super::word::{expect_word, Kind, Word};

pub enum Value {
    Boolean(String),
    Number(String),
    String(String),
    Array(Vec<Value>),
    Reference(String),
}

impl Value {
    pub fn build_from_first_item(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let value = expect_word("Value expected.", &mut iter)?;

        // Value is an array.
        if value.kind == Some(Kind::OpeningBracket) {
            let mut sub_values = Vec::new();

            loop {
                sub_values.push(Self::build_from_first_item(&mut iter)?);

                let delimiter = expect_word("Unexpected end of script.", &mut iter)?;

                if delimiter.kind == Some(Kind::ClosingBracket) {
                    return Ok(Self::Array(sub_values));
                }
                else if delimiter.kind != Some(Kind::Comma) {
                    return Err(ScriptError::new("Unexpected symbol.".to_string(), delimiter.text, delimiter.line, delimiter.line_position, delimiter.absolute_position));
                }
                // Else delimiter_kind is equal to comma, so continue…
            }

        }
        // Value is a single element.
        else {
            match value.kind {
                Some(Kind::Number) => Ok(Self::Number(value.text)),
                Some(Kind::String) => Ok(Self::String(value.text)),
                Some(Kind::Reference) => Ok(Self::Reference(value.text)),
                Some(Kind::Name) => {
                    if value.text == "true" || value.text == "false" {
                        Ok(Self::Boolean(value.text))
                    }
                    else {
                        Err(ScriptError::new("Value expected.".to_string(), value.text, value.line, value.line_position, value.absolute_position))
                    }
                },
                _ => Err(ScriptError::new("Value expected.".to_string(), value.text, value.line, value.line_position, value.absolute_position))
            }
        }
    }
}

