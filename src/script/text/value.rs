
//! Module dedicated to [Value](enum.Value.html) parsing.

use crate::script::error::ScriptError;

use super::word::{expect_word, Kind, Word};

/// Enum describing a textual value.
/// 
/// It sets what kind of value is represented, as well as its associated text.
#[derive(Clone)]
pub enum Value {
    /// `true` or `false`.
    Boolean(String),
    /// Number, see [Kind::Number](../word/enum.Kind.html#variant.Number).
    Number(String),
    /// String, see [Kind::String](../word/enum.Kind.html#variant.String).
    String(String),
    /// Array, representing an arbitrary long vector of values, each of which may be of its own variant kind.
    Array(Vec<Value>),
    /// Name, see [Kind::Name](../word/enum.Kind.html#variant.Name).
    Name(String),
    /// Reference, see [Kind::Reference](../word/enum.Kind.html#variant.Reference).
    Reference(String),
}

impl Value {
    /// Build a value by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the declaration of value.
    /// 
    /// ```
    /// # use lang_trial::script::error::ScriptError;
    /// # use lang_trial::script::text::word::*;
    /// # use lang_trial::script::text::value::Value;
    /// # use std::mem;
    /// let text = r##"
    /// true
    /// -123
    /// "I am a string."
    /// [1, 3, 5, 7]
    /// hereIsName
    /// @hereIsReference
    /// "##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Boolean("".to_string())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Number("".to_string())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::String("".to_string())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Array(vec![])));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Name("".to_string())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Reference("".to_string())));
    /// # Ok::<(), ScriptError>(())
    /// ```
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
                        Ok(Self::Name(value.text))
                    }
                },
                _ => Err(ScriptError::new("Value expected.".to_string(), value.text, value.line, value.line_position, value.absolute_position))
            }
        }
    }
}

