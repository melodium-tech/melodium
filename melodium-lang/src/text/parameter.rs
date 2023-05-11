//! Module dedicated to [Parameter] parsing.

use core::slice::Windows;

use super::r#type::Type;
use super::value::Value;
use super::word::*;
use super::PositionnedString;
use crate::ScriptError;

/// Structure describing a textual parameter.
///
/// It owns a name, and optionnal [Type] and/or [Value]. There is no logical dependency between them at this point.
#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: PositionnedString,
    pub variability: Option<PositionnedString>,
    pub r#type: Option<Type>,
    pub value: Option<Value>,
}

impl Parameter {
    /// Build a parameter by parsing words, starting when name is expected.
    ///
    /// * `variability_or_name`: The variability or name already parsed for the `Parameter` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Type].
    ///
    pub fn build_from_name(
        variability_or_name: PositionnedString,
        mut iter: &mut Windows<Word>,
    ) -> Result<Self, ScriptError> {
        if variability_or_name.string == "var" || variability_or_name.string == "const" {
            match iter.next().map(|s| &s[0]) {
                Some(w) if w.kind == Some(Kind::Name) => {
                    iter.next()
                        .map(|s| &s[0])
                        .ok_or_else(|| ScriptError::end_of_script(55))
                        .and_then(|w| {
                            if w.kind != Some(Kind::Colon) {
                                Err(ScriptError::word(56, w.clone(), &[Kind::Colon]))
                            } else {
                                Ok(())
                            }
                        })?;

                    Self::build_from_type(Some(variability_or_name), w.into(), &mut iter)
                }
                Some(w) => return Err(ScriptError::word(57, w.clone(), &[Kind::Name])),
                None => return Err(ScriptError::end_of_script(58)),
            }
        } else {
            iter.next()
                .map(|s| &s[0])
                .ok_or_else(|| ScriptError::end_of_script(59))
                .and_then(|w| {
                    if w.kind != Some(Kind::Colon) {
                        Err(ScriptError::word(60, w.clone(), &[Kind::Colon]))
                    } else {
                        Ok(())
                    }
                })?;

            Self::build_from_type(None, variability_or_name, &mut iter)
        }
    }

    /// Build a parameter by parsing words, starting when named [Type] is expected.
    ///
    /// * `variability`: The variability already parsed for the `Parameter` (its accuracy is under responsibility of the caller).
    /// * `name`: The name already parsed for the `Parameter` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Type].
    ///
    pub fn build_from_type(
        variability: Option<PositionnedString>,
        name: PositionnedString,
        mut iter: &mut Windows<Word>,
    ) -> Result<Self, ScriptError> {
        let (r#type, possible_equal) = Type::build(&mut iter)?;

        match possible_equal.kind {
            Some(Kind::Equal) => {
                // We discard the equal sign.
                iter.next();

                let value = Value::build_from_first_item(&mut iter)?;

                Ok(Self {
                    name,
                    variability,
                    r#type: Some(r#type),
                    value: Some(value),
                })
            }
            _ => Ok(Self {
                name,
                variability,
                r#type: Some(r#type),
                value: None,
            }),
        }
    }

    /// Build a parameter by parsing words, starting when a [Value] is expected.
    ///
    /// * `name`: The name already parsed for the `Parameter` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Value].
    ///
    pub fn build_from_value(
        name: PositionnedString,
        iter: &mut Windows<Word>,
    ) -> Result<Self, ScriptError> {
        let value = Value::build_from_first_item(iter)?;

        Ok(Self {
            name,
            variability: None,
            r#type: None,
            value: Some(value),
        })
    }
}
