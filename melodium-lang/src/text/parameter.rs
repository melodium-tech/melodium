//! Module dedicated to [Parameter] parsing.

use super::r#type::Type;
use super::value::Value;
use super::word::*;
use super::CommentsAnnotations;
use super::PositionnedString;
use crate::ScriptError;
use core::slice::Windows;
use std::collections::HashMap;

/// Structure describing a textual parameter.
///
/// It owns a name, and optionnal [Type] and/or [Value]. There is no logical dependency between them at this point.
#[derive(Clone, Debug)]
pub struct Parameter {
    pub annotations: Option<CommentsAnnotations>,
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
        annotations: Option<CommentsAnnotations>,
        variability_or_name: PositionnedString,
        mut iter: &mut Windows<Word>,
        global_annotations: &mut HashMap<Word, CommentsAnnotations>,
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

                    Self::build_from_type(
                        annotations,
                        Some(variability_or_name),
                        w.into(),
                        &mut iter,
                        global_annotations,
                    )
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

            Self::build_from_type(
                annotations,
                None,
                variability_or_name,
                &mut iter,
                global_annotations,
            )
        }
    }

    /// Build a parameter by parsing words, starting when named [Type] is expected.
    ///
    /// * `variability`: The variability already parsed for the `Parameter` (its accuracy is under responsibility of the caller).
    /// * `name`: The name already parsed for the `Parameter` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Type].
    ///
    pub fn build_from_type(
        annotations: Option<CommentsAnnotations>,
        variability: Option<PositionnedString>,
        name: PositionnedString,
        mut iter: &mut Windows<Word>,
        global_annotations: &mut HashMap<Word, CommentsAnnotations>,
    ) -> Result<Self, ScriptError> {
        let (r#type, possible_equal) = Type::build(&mut iter, global_annotations)?;

        match possible_equal.kind {
            Some(Kind::Equal) => {
                // We discard the equal sign.
                iter.next();

                let value = Value::build_from_first_item(&mut iter, global_annotations)?;

                Ok(Self {
                    annotations,
                    name,
                    variability,
                    r#type: Some(r#type),
                    value: Some(value),
                })
            }
            _ => Ok(Self {
                annotations,
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
        annotations: Option<CommentsAnnotations>,
        name: PositionnedString,
        iter: &mut Windows<Word>,
        global_annotations: &mut HashMap<Word, CommentsAnnotations>,
    ) -> Result<Self, ScriptError> {
        let value = Value::build_from_first_item(iter, global_annotations)?;

        Ok(Self {
            annotations,
            name,
            variability: None,
            r#type: None,
            value: Some(value),
        })
    }
}
