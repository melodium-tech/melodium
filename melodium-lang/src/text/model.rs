//! Module dedicated to [Model] parsing.

use core::slice::Windows;
use std::collections::HashMap;

use super::common::parse_parameters_declarations;
use super::parameter::Parameter;
use super::word::{Kind, Word};
use super::{CommentsAnnotations, PositionnedString};
use crate::ScriptError;

/// Structure describing a textual model.
///
/// It owns a name, parameters, and a type (model type, not [data type](super::Type)).
#[derive(Clone, Debug)]
pub struct Model {
    pub annotations: Option<CommentsAnnotations>,
    pub name: PositionnedString,
    pub parameters: Vec<Parameter>,
    pub r#type: PositionnedString,
    pub assignations: Vec<Parameter>,
}

impl Model {
    /// Build a model by parsing words.
    ///
    /// * `iter`: Iterator over words list, next() being expected to be the name.
    ///
    pub fn build(
        mut iter: &mut Windows<Word>,
        mut self_annotations: Option<CommentsAnnotations>,
        global_annotations: &mut HashMap<Word, CommentsAnnotations>,
    ) -> Result<Self, ScriptError> {
        let word_name = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(62))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(63, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.clone())
                }
            })?;
        let name: PositionnedString = (&word_name).into();

        let parameters;
        match iter.next().map(|s| &s[0]) {
            Some(w) if w.kind == Some(Kind::OpeningParenthesis) => {
                parameters = parse_parameters_declarations(&mut iter, global_annotations)?;
            }
            Some(w) => {
                return Err(ScriptError::word(
                    64,
                    w.clone(),
                    &[Kind::OpeningParenthesis],
                ))
            }
            None => return Err(ScriptError::end_of_script(65)),
        }

        iter.next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(66))
            .and_then(|w| {
                if w.kind != Some(Kind::Colon) {
                    Err(ScriptError::word(67, w.clone(), &[Kind::Colon]))
                } else {
                    Ok(())
                }
            })?;
        let r#type = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(68))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(69, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.into())
                }
            })?;

        iter.next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(70))
            .and_then(|w| {
                if w.kind != Some(Kind::OpeningBrace) {
                    Err(ScriptError::word(71, w.clone(), &[Kind::OpeningBrace]))
                } else {
                    Ok(())
                }
            })?;

        let mut assignations = Vec::new();

        loop {
            match iter.next().map(|s| &s[0]) {
                Some(w) if w.kind == Some(Kind::ClosingBrace) => break,
                Some(w) if w.kind == Some(Kind::Name) => {
                    iter.next()
                        .map(|s| &s[0])
                        .ok_or_else(|| ScriptError::end_of_script(72))
                        .and_then(|w| {
                            if w.kind != Some(Kind::Equal) {
                                Err(ScriptError::word(73, w.clone(), &[Kind::Equal]))
                            } else {
                                Ok(())
                            }
                        })?;
                    assignations.push(Parameter::build_from_value(
                        global_annotations.remove(w),
                        w.into(),
                        &mut iter,
                    )?);
                }
                Some(w) => {
                    return Err(ScriptError::word(
                        74,
                        w.clone(),
                        &[Kind::Name, Kind::ClosingBrace],
                    ))
                }
                None => return Err(ScriptError::end_of_script(75)),
            }
        }

        if let Some(doc) = self_annotations.as_mut().and_then(|sa| sa.doc.as_mut()) {
            doc.remove_indent();
        }

        Ok(Self {
            annotations: self_annotations,
            name,
            parameters,
            r#type,
            assignations,
        })
    }
}
