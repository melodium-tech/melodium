
//! Module dedicated to [Value](enum.Value.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::{expect_word, expect_word_kind, Kind, Word};

/// Enum describing a textual value.
/// 
/// It sets what kind of value is represented, as well as its associated text.
#[derive(Clone)]
pub enum Value {
    /// `true` or `false`.
    Boolean(PositionnedString),
    /// Number, see [Kind::Number](../word/enum.Kind.html#variant.Number).
    Number(PositionnedString),
    /// String, see [Kind::String](../word/enum.Kind.html#variant.String).
    String(PositionnedString),
    /// Array, representing an arbitrary long vector of values, each of which may be of its own variant kind.
    Array(Vec<Value>),
    /// Name, see [Kind::Name](../word/enum.Kind.html#variant.Name).
    Name(PositionnedString),
    /// Reference, see [Kind::Reference](../word/enum.Kind.html#variant.Reference).
    /// First element being the reference itself, second element the inner refered component.
    /// `@Foo[bar]`: (`@Foo`, `bar`)
    Reference((PositionnedString, PositionnedString)),
}

impl Value {
    /// Build a value by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the declaration of value.
    /// 
    /// ```
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::word::*;
    /// # use melodium_rust::script::text::value::Value;
    /// # use std::mem;
    /// let text = r##"
    /// true
    /// -123
    /// "I am a string."
    /// [1, 3, 5, 7]
    /// hereIsName
    /// @HereIsReference[toSomething]
    /// "##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Boolean(PositionnedString::default())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Number(PositionnedString::default())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::String(PositionnedString::default())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Array(vec![])));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Name(PositionnedString::default())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Reference((PositionnedString::default(), PositionnedString::default()))));
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
                    return Err(ScriptError::word("Unexpected symbol.".to_string(), delimiter.text, delimiter.position));
                }
                // Else delimiter_kind is equal to comma, so continue…
            }

        }
        // Value is a reference.
        else if value.kind == Some(Kind::Reference) {

            let reference = value;

            expect_word_kind(Kind::OpeningBracket, "Opening bracket '[' expected.", &mut iter)?;
            let inner_reference = expect_word_kind(Kind::Name, "Element name expected.", &mut iter)?;
            expect_word_kind(Kind::ClosingBracket, "Closing bracket ']' expected.", &mut iter)?;

            Ok(Self::Reference((
                PositionnedString { string: reference.text, position: reference.position},
                inner_reference
            )))
        }
        // Value is a single element.
        else {
            match value.kind {
                Some(Kind::Number) => Ok(Self::Number(PositionnedString { string: value.text, position: value.position})),
                Some(Kind::String) => Ok(Self::String(PositionnedString { string: value.text, position: value.position})),
                Some(Kind::Name) => {
                    if value.text == "true" || value.text == "false" {
                        Ok(Self::Boolean(PositionnedString { string: value.text, position: value.position}))
                    }
                    else {
                        Ok(Self::Name(PositionnedString { string: value.text, position: value.position}))
                    }
                },
                _ => Err(ScriptError::word("Value expected.".to_string(), value.text, value.position))
            }
        }
    }
}

