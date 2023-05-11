//! Module dedicated to [Value] parsing.

use core::slice::Windows;

use super::word::{Kind, Word};
use super::{Function, Position, PositionnedString};
use crate::ScriptError;

/// Enum describing a textual value.
///
/// It sets what kind of value is represented, as well as its associated text.
#[derive(Clone, Debug)]
pub enum Value {
    /// `true` or `false`.
    Boolean(PositionnedString),
    /// Number, see [Kind::Number].
    Number(PositionnedString),
    /// String, see [Kind::String].
    String(PositionnedString),
    /// Char, see [Kind::Char].
    Character(PositionnedString),
    /// Byte, see [Kind::Byte].
    Byte(PositionnedString),
    /// Array, representing an arbitrary long vector of values, each of which may be of its own variant kind.
    Array(PositionnedString, Vec<Value>),
    /// Name, see [Kind::Name].
    Name(PositionnedString),
    /// ContextReference, see [Kind::Context].
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
    pub fn build_from_first_item(mut iter: &mut Windows<Word>) -> Result<Self, ScriptError> {
        match iter.next().map(|s| &s[0]) {
            Some(w) if w.kind == Some(Kind::OpeningBracket) => {
                let mut sub_values = Vec::new();

                loop {
                    sub_values.push(Self::build_from_first_item(&mut iter)?);

                    match iter.next().map(|s| &s[0]) {
                        Some(delimiter) if delimiter.kind == Some(Kind::ClosingBracket) => {
                            return Ok(Self::Array(
                                PositionnedString {
                                    string: delimiter.text.clone(),
                                    position: delimiter.position,
                                },
                                sub_values,
                            ));
                        }
                        Some(delimiter) if delimiter.kind == Some(Kind::Comma) => continue,
                        Some(w) => {
                            return Err(ScriptError::word(
                                3,
                                w.clone(),
                                &[Kind::Comma, Kind::ClosingBracket],
                            ));
                        }
                        None => return Err(ScriptError::end_of_script(4)),
                    }

                    // Else delimiter_kind is equal to comma, so continue…
                }
            }
            Some(w) if w.kind == Some(Kind::Context) => {
                let context = w.into();

                iter.next()
                    .map(|s| &s[0])
                    .ok_or_else(|| ScriptError::end_of_script(5))
                    .and_then(|w| {
                        if w.kind != Some(Kind::OpeningBracket) {
                            Err(ScriptError::word(6, w.clone(), &[Kind::OpeningBracket]))
                        } else {
                            Ok(())
                        }
                    })?;

                let inner_reference = iter
                    .next()
                    .map(|s| &s[0])
                    .ok_or_else(|| ScriptError::end_of_script(7))
                    .and_then(|w| {
                        if w.kind != Some(Kind::Name) {
                            Err(ScriptError::word(8, w.clone(), &[Kind::Name]))
                        } else {
                            Ok(w.into())
                        }
                    })?;

                iter.next()
                    .map(|s| &s[0])
                    .ok_or_else(|| ScriptError::end_of_script(9))
                    .and_then(|w| {
                        if w.kind != Some(Kind::ClosingBracket) {
                            Err(ScriptError::word(10, w.clone(), &[Kind::ClosingBracket]))
                        } else {
                            Ok(())
                        }
                    })?;

                Ok(Self::ContextReference((context, inner_reference)))
            }
            Some(w) if w.kind == Some(Kind::Function) => {
                let function = Function::build_from_parameters(w.into(), &mut iter)?;

                Ok(Self::Function(function))
            }
            Some(value) => match value.kind {
                Some(Kind::Number) => Ok(Self::Number(PositionnedString {
                    string: value.text.clone(),
                    position: value.position,
                })),
                Some(Kind::String) => Ok(Self::String(PositionnedString {
                    string: value.text.clone(),
                    position: value.position,
                })),
                Some(Kind::Character) => Ok(Self::Character(PositionnedString {
                    string: value.text.clone(),
                    position: value.position,
                })),
                Some(Kind::Byte) => Ok(Self::Byte(PositionnedString {
                    string: value.text.clone(),
                    position: value.position,
                })),
                Some(Kind::Name) => {
                    if value.text == "true" || value.text == "false" {
                        Ok(Self::Boolean(PositionnedString {
                            string: value.text.clone(),
                            position: value.position,
                        }))
                    } else {
                        Ok(Self::Name(PositionnedString {
                            string: value.text.clone(),
                            position: value.position,
                        }))
                    }
                }
                _ => Err(ScriptError::word(
                    2,
                    value.clone(),
                    &[
                        Kind::Number,
                        Kind::String,
                        Kind::Character,
                        Kind::Byte,
                        Kind::Name,
                    ],
                )),
            },
            None => Err(ScriptError::end_of_script(1)),
        }
    }

    pub fn get_positionned_string(&self) -> &PositionnedString {
        match self {
            Value::Boolean(ps) => ps,
            Value::Number(ps) => ps,
            Value::String(ps) => ps,
            Value::Character(ps) => ps,
            Value::Byte(ps) => ps,
            Value::Array(ps, _) => ps,
            Value::Name(ps) => ps,
            Value::ContextReference((ps, _)) => ps,
            Value::Function(func) => &func.name,
        }
    }

    pub fn get_position(&self) -> Position {
        self.get_positionned_string().position.clone()
    }
}
