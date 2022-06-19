
//! Module dedicated to [Value](enum.Value.html) parsing.

use crate::script::error::ScriptError;

use super::{PositionnedString, Position, Function};
use super::word::{expect_word, expect_word_kind, Kind, Word};

/// Enum describing a textual value.
/// 
/// It sets what kind of value is represented, as well as its associated text.
#[derive(Clone, Debug)]
pub enum Value {
    /// `true` or `false`.
    Boolean(PositionnedString),
    /// Number, see [Kind::Number](../word/enum.Kind.html#variant.Number).
    Number(PositionnedString),
    /// String, see [Kind::String](../word/enum.Kind.html#variant.String).
    String(PositionnedString),
    /// Char, see [Kind::Char](../word/enum.Kind.html#variant.Char).
    Character(PositionnedString),
    /// Byte, see [Kind::Byte](../word/enum.Kind.html#variant.Byte).
    Byte(PositionnedString),
    /// Array, representing an arbitrary long vector of values, each of which may be of its own variant kind.
    Array(PositionnedString, Vec<Value>),
    /// Name, see [Kind::Name](../word/enum.Kind.html#variant.Name).
    Name(PositionnedString),
    /// ContextReference, see [Kind::Context](../word/enum.Kind.html#variant.Context).
    /// First element being the context itself, second element the inner refered component.
    /// `@Foo[bar]`: (`@Foo`, `bar`)
    ContextReference((PositionnedString, PositionnedString)),
    /// Function, representing a function call.
    Function(Function),
}

impl Value {
    /// Build a value by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the declaration of value.
    /// 
    /// ```
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::word::*;
    /// # use melodium::script::text::value::Value;
    /// # use melodium::script::text::function::Function;
    /// # use std::mem;
    /// let text = r##"
    /// true
    /// -123
    /// "I am a string."
    /// [1, 3, 5, 7]
    /// hereIsName
    /// @HereIsReference[toSomething]
    /// |hereIsFunction()
    /// |hereIsFunctionWithParameters(45, 46, 47, "Foo", "Bar", true)
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
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Array(PositionnedString::default(), vec![])));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Name(PositionnedString::default())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::ContextReference((PositionnedString::default(), PositionnedString::default()))));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Function(Function::default())));
    /// 
    /// let value = Value::build_from_first_item(&mut iter)?;
    /// assert_eq!(mem::discriminant(&value), mem::discriminant(&Value::Function(Function::default())));
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
                    return Ok(Self::Array(
                        PositionnedString { string: delimiter.text, position: delimiter.position} ,
                        sub_values
                    ));
                }
                else if delimiter.kind != Some(Kind::Comma) {
                    return Err(ScriptError::word("Unexpected symbol.".to_string(), delimiter.text, delimiter.position));
                }
                // Else delimiter_kind is equal to comma, so continue…
            }

        }
        // Value is a context (so a reference to something in it).
        else if value.kind == Some(Kind::Context) {

            let context = value;

            expect_word_kind(Kind::OpeningBracket, "Opening bracket '[' expected.", &mut iter)?;
            let inner_reference = expect_word_kind(Kind::Name, "Element name expected.", &mut iter)?;
            expect_word_kind(Kind::ClosingBracket, "Closing bracket ']' expected.", &mut iter)?;

            Ok(Self::ContextReference((
                PositionnedString { string: context.text, position: context.position},
                inner_reference
            )))
        }
        // Value is a function call.
        else if value.kind == Some(Kind::Function) {

            let function = Function::build_from_parameters(PositionnedString { string: value.text, position: value.position}, &mut iter)?;

            Ok(Self::Function(function))
        }
        // Value is a single element.
        else {
            match value.kind {
                Some(Kind::Number) => Ok(Self::Number(PositionnedString { string: value.text, position: value.position})),
                Some(Kind::String) => Ok(Self::String(PositionnedString { string: value.text, position: value.position})),
                Some(Kind::Character) => Ok(Self::Character(PositionnedString { string: value.text, position: value.position})),
                Some(Kind::Byte) => Ok(Self::Byte(PositionnedString { string: value.text, position: value.position})),
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

    pub fn get_position(&self) -> Position {
        
        match self {
            Value::Boolean(ps) => ps.position,
            Value::Number(ps) => ps.position,
            Value::String(ps) => ps.position,
            Value::Character(ps) => ps.position,
            Value::Byte(ps) => ps.position,
            Value::Array(ps, _) => ps.position,
            Value::Name(ps) => ps.position,
            Value::ContextReference((ps, _)) => ps.position,
            Value::Function(func) => func.name.position,
        }
    }
}

