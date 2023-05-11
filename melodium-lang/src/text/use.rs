//! Module dedicated to [Use] parsing.

use core::slice::Windows;

use super::word::*;
use super::PositionnedString;
use crate::ScriptError;

/// Structure describing a textual use.
///
/// It owns the path, as vector of strings (which were separated by slashes `/`), the used element name, and optionally the alias. There is no logical nor existence check at this point.
#[derive(Clone, Debug)]
pub struct Use {
    pub path: Vec<PositionnedString>,
    pub element: PositionnedString,
    pub r#as: Option<PositionnedString>,
}

impl Use {
    /// Build use by parsing words.
    ///
    /// * `iter`: Iterator over words list, next() being expected to be the beginning of the path.
    ///
    pub fn build(iter: &mut Windows<Word>) -> Result<Self, ScriptError> {
        let mut path = Vec::new();
        let element;
        let use_as;

        loop {
            match iter.next().map(|s| &s[0]) {
                Some(w) if w.kind == Some(Kind::Name) => path.push(w.into()),
                Some(w) => return Err(ScriptError::word(12, w.clone(), &[Kind::Name])),
                None => return Err(ScriptError::end_of_script(11)),
            }

            match iter.next().map(|s| &s[0]) {
                Some(w) if w.kind == Some(Kind::Slash) => continue,
                Some(w) if w.kind == Some(Kind::Colon) => {
                    iter.next()
                        .map(|s| &s[0])
                        .ok_or_else(|| ScriptError::end_of_script(15))
                        .and_then(|w| {
                            if w.kind != Some(Kind::Colon) {
                                Err(ScriptError::word(16, w.clone(), &[Kind::Colon]))
                            } else {
                                Ok(())
                            }
                        })?;

                    let expected_kind;
                    let name = iter.next();
                    match name.map(|s| &s[0]) {
                        Some(w) if w.kind == Some(Kind::Name) => {
                            element = w.into();
                            expected_kind = Kind::Name;
                        }
                        Some(w) if w.kind == Some(Kind::Function) => {
                            element = w.into();
                            expected_kind = Kind::Function;
                        }
                        Some(w) if w.kind == Some(Kind::Context) => {
                            element = w.into();
                            expected_kind = Kind::Context;
                        }
                        Some(w) => {
                            return Err(ScriptError::word(
                                17,
                                w.clone(),
                                &[Kind::Name, Kind::Function, Kind::Context],
                            ))
                        }
                        None => return Err(ScriptError::end_of_script(18)),
                    }

                    match name.map(|s| &s[1]) {
                        Some(w) if w.kind == Some(Kind::Name) && w.text == "as" => {
                            iter.next(); // Skipping "as"
                            match iter.next().map(|s| &s[0]) {
                                Some(w) if w.kind == Some(expected_kind) => use_as = Some(w.into()),
                                _ => {
                                    return Err(ScriptError::word(
                                        20,
                                        w.clone(),
                                        match expected_kind {
                                            Kind::Name => &[Kind::Name],
                                            Kind::Context => &[Kind::Context],
                                            Kind::Function => &[Kind::Function],
                                            _ => &[],
                                        },
                                    ))
                                }
                            }
                        }
                        _ => {
                            use_as = None;
                        }
                    }

                    break;
                }
                Some(w) => {
                    return Err(ScriptError::word(
                        14,
                        w.clone(),
                        &[Kind::Slash, Kind::Colon],
                    ))
                }
                None => return Err(ScriptError::end_of_script(13)),
            }
        }

        Ok(Self {
            path,
            element,
            r#as: use_as,
        })
    }
}
