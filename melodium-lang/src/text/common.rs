//! Contains convenience functions and tools for text parsing.

use super::parameter::Parameter;
use super::word::{Kind, Word};
use super::{CommentsAnnotations, Generic};
use crate::ScriptError;
use core::slice::Windows;
use std::collections::HashMap;

/// Build a parameter declaration list by parsing words.
///
/// * `iter`: Iterator over words list, next() being expected to be the first parameter, _not_ parenthesis.
///
pub fn parse_parameters_declarations(
    mut iter: &mut Windows<Word>,
    global_annotations: &mut HashMap<Word, CommentsAnnotations>,
) -> Result<Vec<Parameter>, ScriptError> {
    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {
        match iter.next().map(|s| &s[0]) {
            Some(w) if w.kind == Some(Kind::ClosingParenthesis) && first_param => break,
            Some(w) if w.kind == Some(Kind::Name) => {
                first_param = false;

                parameters.push(Parameter::build_from_name(
                    global_annotations.remove(w),
                    w.into(),
                    &mut iter,
                    global_annotations,
                )?);

                match iter.next().map(|s| &s[0]) {
                    Some(w) if w.kind == Some(Kind::Comma) => continue,
                    Some(w) if w.kind == Some(Kind::ClosingParenthesis) => break,
                    Some(w) => {
                        return Err(ScriptError::word(
                            87,
                            w.clone(),
                            &[Kind::Comma, Kind::ClosingParenthesis],
                        ))
                    }
                    None => return Err(ScriptError::end_of_script(88)),
                }
            }
            Some(w) => {
                return Err(ScriptError::word(
                    85,
                    w.clone(),
                    &[Kind::Name, Kind::ClosingParenthesis],
                ))
            }
            None => return Err(ScriptError::end_of_script(86)),
        }
    }

    Ok(parameters)
}

/// Build a parameter assignations list by parsing words.
///
/// * `iter`: Iterator over words list, next() being expected to be the the first parameter, _not_ parenthesis.
pub fn parse_parameters_assignations(
    mut iter: &mut Windows<Word>,
    global_annotations: &mut HashMap<Word, CommentsAnnotations>,
) -> Result<Vec<Parameter>, ScriptError> {
    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {
        match iter.next().map(|s| &s[0]) {
            Some(w) if w.kind == Some(Kind::ClosingParenthesis) && first_param => break,
            Some(w) if w.kind == Some(Kind::Name) => {
                first_param = false;

                iter.next()
                    .map(|s| &s[0])
                    .ok_or_else(|| ScriptError::end_of_script(89))
                    .and_then(|w| {
                        if w.kind != Some(Kind::Equal) {
                            Err(ScriptError::word(90, w.clone(), &[Kind::Equal]))
                        } else {
                            Ok(())
                        }
                    })?;

                parameters.push(Parameter::build_from_value(
                    global_annotations.remove(w),
                    w.into(),
                    &mut iter,
                    global_annotations,
                )?);

                match iter.next().map(|s| &s[0]) {
                    Some(w) if w.kind == Some(Kind::Comma) => continue,
                    Some(w) if w.kind == Some(Kind::ClosingParenthesis) => break,
                    Some(w) => {
                        return Err(ScriptError::word(
                            91,
                            w.clone(),
                            &[Kind::Comma, Kind::ClosingParenthesis],
                        ))
                    }
                    None => return Err(ScriptError::end_of_script(92)),
                }
            }
            Some(w) => {
                return Err(ScriptError::word(
                    93,
                    w.clone(),
                    &[Kind::Name, Kind::ClosingParenthesis],
                ))
            }
            None => return Err(ScriptError::end_of_script(94)),
        }
    }

    Ok(parameters)
}

/// Build a configuration declaration list by parsing words.
///
/// * `iter`: Iterator over words list, next() being expected to be the first parameter, _not_ bracket.
pub fn parse_configuration_declarations(
    mut iter: &mut Windows<Word>,
    global_annotations: &mut HashMap<Word, CommentsAnnotations>,
) -> Result<Vec<Parameter>, ScriptError> {
    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {
        match iter.next().map(|s| &s[0]) {
            Some(w) if w.kind == Some(Kind::ClosingBracket) && first_param => break,
            Some(w) if w.kind == Some(Kind::Name) => {
                first_param = false;

                iter.next()
                    .map(|s| &s[0])
                    .ok_or_else(|| ScriptError::end_of_script(95))
                    .and_then(|w| {
                        if w.kind != Some(Kind::Colon) {
                            Err(ScriptError::word(96, w.clone(), &[Kind::Colon]))
                        } else {
                            Ok(())
                        }
                    })?;

                parameters.push(Parameter::build_from_type(
                    global_annotations.remove(w),
                    None,
                    w.into(),
                    &mut iter,
                    global_annotations,
                )?);

                match iter.next().map(|s| &s[0]) {
                    Some(w) if w.kind == Some(Kind::Comma) => continue,
                    Some(w) if w.kind == Some(Kind::ClosingBracket) => break,
                    Some(w) => {
                        return Err(ScriptError::word(
                            97,
                            w.clone(),
                            &[Kind::Comma, Kind::ClosingBracket],
                        ))
                    }
                    None => return Err(ScriptError::end_of_script(98)),
                }
            }
            Some(w) => {
                return Err(ScriptError::word(
                    99,
                    w.clone(),
                    &[Kind::Name, Kind::ClosingBracket],
                ))
            }
            None => return Err(ScriptError::end_of_script(100)),
        }
    }

    Ok(parameters)
}

/// Build a configuration assignation list by parsing words.
///
/// * `iter`: Iterator over words list, next() being expected to be the first parameter, _not_ bracket.
pub fn parse_configuration_assignations(
    mut iter: &mut Windows<Word>,
    global_annotations: &mut HashMap<Word, CommentsAnnotations>,
) -> Result<Vec<Parameter>, ScriptError> {
    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {
        match iter.next().map(|s| &s[0]) {
            Some(w) if w.kind == Some(Kind::ClosingBracket) && first_param => break,
            Some(w) if w.kind == Some(Kind::Name) => {
                first_param = false;

                iter.next()
                    .map(|s| &s[0])
                    .ok_or_else(|| ScriptError::end_of_script(101))
                    .and_then(|w| {
                        if w.kind != Some(Kind::Equal) {
                            Err(ScriptError::word(102, w.clone(), &[Kind::Equal]))
                        } else {
                            Ok(())
                        }
                    })?;

                parameters.push(Parameter::build_from_value(
                    global_annotations.remove(w),
                    w.into(),
                    &mut iter,
                    global_annotations,
                )?);

                match iter.next().map(|s| &s[0]) {
                    Some(w) if w.kind == Some(Kind::Comma) => continue,
                    Some(w) if w.kind == Some(Kind::ClosingBracket) => break,
                    Some(w) => {
                        return Err(ScriptError::word(
                            103,
                            w.clone(),
                            &[Kind::Comma, Kind::ClosingBracket],
                        ))
                    }
                    None => return Err(ScriptError::end_of_script(104)),
                }
            }
            Some(w) => {
                return Err(ScriptError::word(
                    105,
                    w.clone(),
                    &[Kind::Name, Kind::ClosingBracket],
                ))
            }
            None => return Err(ScriptError::end_of_script(106)),
        }
    }

    Ok(parameters)
}

/// Build a generics declaration/assignation list by parsing words.
///
/// * `iter`: Iterator over words list, next() being expected to be the first parameter, _not_ bracket.
pub fn parse_generics(
    iter: &mut Windows<Word>,
    global_annotations: &mut HashMap<Word, CommentsAnnotations>,
) -> Result<Vec<Generic>, ScriptError> {
    let mut generics = Vec::new();

    let mut first_generic = true;
    loop {
        match iter.next().map(|s| &s[0]) {
            Some(w) if w.kind == Some(Kind::ClosingChevron) && first_generic => break,
            Some(w) if w.kind == Some(Kind::Name) => {
                first_generic = false;

                generics.push(Generic {
                    annotations: global_annotations.remove(w),
                    name: w.into(),
                });

                match iter.next().map(|s| &s[0]) {
                    Some(w) if w.kind == Some(Kind::Comma) => continue,
                    Some(w) if w.kind == Some(Kind::ClosingChevron) => break,
                    Some(w) => {
                        return Err(ScriptError::word(
                            158,
                            w.clone(),
                            &[Kind::Comma, Kind::ClosingChevron],
                        ))
                    }
                    None => return Err(ScriptError::end_of_script(159)),
                }
            }
            Some(w) => {
                return Err(ScriptError::word(
                    160,
                    w.clone(),
                    &[Kind::Name, Kind::ClosingChevron],
                ))
            }
            None => return Err(ScriptError::end_of_script(161)),
        }
    }

    Ok(generics)
}
