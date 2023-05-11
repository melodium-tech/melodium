//! Module dedicated to [Connection] parsing.

use core::slice::Windows;

use super::word::{Kind, Word};
use super::PositionnedString;
use crate::ScriptError;

/// Structure describing a textual connection.
///
/// It owns starting point and ending point names, and the names of data associated with, if any.
#[derive(Clone, Debug)]
pub struct Connection {
    pub name_start_point: PositionnedString,
    pub name_data_out: Option<PositionnedString>,
    pub name_end_point: PositionnedString,
    pub name_data_in: Option<PositionnedString>,
}

impl Connection {
    /// Build a connection by parsing words, starting when then end point name is expected.
    ///
    /// * `name`: The name already parsed for the start point (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be the end point name.
    ///
    pub fn build_from_name_end_point(
        name: PositionnedString,
        iter: &mut Windows<Word>,
    ) -> Result<Self, ScriptError> {
        let name_end_point = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(77))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(78, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.into())
                }
            })?;

        Ok(Self {
            name_start_point: name,
            name_data_out: None,
            name_end_point,
            name_data_in: None,
        })
    }

    /// Build a connection by parsing words, starting when then name of data out is expected.
    ///
    /// * `name`: The name already parsed for the data out (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be the data out name.
    ///
    pub fn build_from_name_data_out(
        name: PositionnedString,
        iter: &mut Windows<Word>,
    ) -> Result<Self, ScriptError> {
        let name_data_out = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(79))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(80, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.into())
                }
            })?;

        iter.next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(85))
            .and_then(|w| {
                if w.kind != Some(Kind::RightArrow) {
                    Err(ScriptError::word(86, w.clone(), &[Kind::RightArrow]))
                } else {
                    Ok(())
                }
            })?;

        let name_end_point = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(81))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(82, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.into())
                }
            })?;

        iter.next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(87))
            .and_then(|w| {
                if w.kind != Some(Kind::Dot) {
                    Err(ScriptError::word(88, w.clone(), &[Kind::Dot]))
                } else {
                    Ok(())
                }
            })?;

        let name_data_in = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(83))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(84, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.into())
                }
            })?;

        Ok(Self {
            name_start_point: name,
            name_data_out: Some(name_data_out),
            name_end_point: name_end_point,
            name_data_in: Some(name_data_in),
        })
    }
}
