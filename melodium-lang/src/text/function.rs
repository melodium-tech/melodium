//! Module dedicated to [Function] parsing.

use super::common::parse_generics;
use super::word::{Kind, Word};
use super::{CommentsAnnotations, Generic, PositionnedString, Value};
use crate::ScriptError;
use core::slice::Windows;
use std::collections::HashMap;

/// Structure describing a textual requirement.
///
/// It owns the requirement name.
#[derive(Clone, Debug, Default)]
pub struct Function {
    pub name: PositionnedString,
    pub generics: Vec<Generic>,
    pub parameters: Vec<Value>,
}

impl Function {
    pub fn build_from_generics(
        name: PositionnedString,
        mut iter: &mut Windows<Word>,
        global_annotations: &mut HashMap<Word, CommentsAnnotations>,
    ) -> Result<Self, ScriptError> {

        // Discard '<'
        iter.next();

        let generics = parse_generics(&mut iter, global_annotations)?;

        Self::build_from_parameters(name, generics, &mut iter, global_annotations)
    }

    pub fn build_from_parameters(
        name: PositionnedString,
        generics: Vec<Generic>,
        mut iter: &mut Windows<Word>,
        global_annotations: &mut HashMap<Word, CommentsAnnotations>,
    ) -> Result<Self, ScriptError> {
        let mut parameters = Vec::new();

        let possible_closing_parenthesis;
        match iter.next().map(|s| (&s[0], &s[1])) {
            Some((w, nw)) if w.kind == Some(Kind::OpeningParenthesis) => {
                possible_closing_parenthesis = Some(nw);
            }
            Some((w, _)) => {
                return Err(ScriptError::word(
                    74,
                    w.clone(),
                    &[Kind::OpeningParenthesis],
                ))
            }
            None => return Err(ScriptError::end_of_script(73)),
        }

        match possible_closing_parenthesis {
            Some(w) if w.kind == Some(Kind::ClosingParenthesis) => {
                // Discard ')'
                iter.next();
            }
            _ => loop {
                parameters.push(Value::build_from_first_item(&mut iter, global_annotations)?);

                match iter.next().map(|s| &s[0]) {
                    Some(w) if w.kind == Some(Kind::Comma) => continue,
                    Some(w) if w.kind == Some(Kind::ClosingParenthesis) => break,
                    Some(w) => {
                        return Err(ScriptError::word(
                            75,
                            w.clone(),
                            &[Kind::Comma, Kind::ClosingParenthesis],
                        ))
                    }
                    None => return Err(ScriptError::end_of_script(76)),
                }
            },
        }

        Ok(Self {
            name,
            generics,
            parameters,
        })
    }
}
