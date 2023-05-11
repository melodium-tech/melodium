//! Module dedicated to [Instanciation] parsing.

use super::common::{parse_configuration_assignations, parse_parameters_assignations};
use super::parameter::Parameter;
use super::word::{Kind, Word};
use super::PositionnedString;
use crate::ScriptError;
use core::slice::Windows;

/// Structure describing a textual instanciation.
///
/// This match the conceptual syntax of calling a model or treatment.
/// It owns a name, a type (treatment or model type, not [data type](super::Type)), and list of [parameters](super::Parameter).
#[derive(Clone, Debug)]
pub struct Instanciation {
    pub name: PositionnedString,
    pub r#type: PositionnedString,
    pub configuration: Vec<Parameter>,
    pub parameters: Vec<Parameter>,
}

impl Instanciation {
    /// Build an instanciation by parsing words.
    ///
    /// * `iter`: Iterator over words list, next() being expected to be the name.
    ///
    pub fn build(mut iter: &mut Windows<Word>) -> Result<Self, ScriptError> {
        let name: PositionnedString = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(62))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(63, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.into())
                }
            })?;

        match iter.next().map(|s| &s[0]) {
            Some(w) if w.kind == Some(Kind::Colon) => {
                Self::build_from_type(name.clone(), &mut iter)
            }
            Some(w) if w.kind == Some(Kind::OpeningBracket) => {
                Self::build_from_configuration(name.clone(), name, &mut iter)
            }
            Some(w) if w.kind == Some(Kind::OpeningParenthesis) => {
                Self::build_from_parameters(name.clone(), name, Vec::new(), &mut iter)
            }
            Some(w) => {
                return Err(ScriptError::word(
                    64,
                    w.clone(),
                    &[Kind::Colon, Kind::OpeningBracket, Kind::OpeningParenthesis],
                ))
            }
            None => return Err(ScriptError::end_of_script(65)),
        }
    }

    /// Build an instanciation by parsing words, starting when its type is expected.
    ///
    /// * `name`: The name already parsed for the `Instanciation` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be the type name.
    ///
    pub fn build_from_type(
        name: PositionnedString,
        mut iter: &mut Windows<Word>,
    ) -> Result<Self, ScriptError> {
        let r#type = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(66))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(67, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.into())
                }
            })?;

        match iter.next().map(|s| &s[0]) {
            Some(w) if w.kind == Some(Kind::OpeningBracket) => {
                Self::build_from_configuration(name.clone(), r#type, &mut iter)
            }
            Some(w) if w.kind == Some(Kind::OpeningParenthesis) => {
                Self::build_from_parameters(name.clone(), r#type, Vec::new(), &mut iter)
            }
            Some(w) => {
                return Err(ScriptError::word(
                    68,
                    w.clone(),
                    &[Kind::OpeningBracket, Kind::OpeningParenthesis],
                ))
            }
            None => return Err(ScriptError::end_of_script(69)),
        }
    }

    /// Build an instanciation by parsing words, starting when configuration [Parameter] is expected.
    ///
    /// * `name`: The name already parsed for the `Instanciation` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Parameter].
    ///
    pub fn build_from_configuration(
        name: PositionnedString,
        r#type: PositionnedString,
        mut iter: &mut Windows<Word>,
    ) -> Result<Self, ScriptError> {
        let configuration = parse_configuration_assignations(&mut iter)?;

        // We expect parameters in any cases.
        iter.next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(70))
            .and_then(|w| {
                if w.kind != Some(Kind::OpeningParenthesis) {
                    Err(ScriptError::word(
                        71,
                        w.clone(),
                        &[Kind::OpeningParenthesis],
                    ))
                } else {
                    Ok(())
                }
            })?;
        Self::build_from_parameters(name, r#type, configuration, &mut iter)
    }

    /// Build an instanciation by parsing words, starting when [Parameter] is expected.
    ///
    /// * `name`: The name already parsed for the `Instanciation` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Parameter].
    ///
    pub fn build_from_parameters(
        name: PositionnedString,
        r#type: PositionnedString,
        configuration: Vec<Parameter>,
        mut iter: &mut Windows<Word>,
    ) -> Result<Self, ScriptError> {
        let parameters = parse_parameters_assignations(&mut iter)?;

        Ok(Self {
            name,
            r#type,
            configuration,
            parameters,
        })
    }
}
