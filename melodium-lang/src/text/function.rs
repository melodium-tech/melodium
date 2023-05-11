//! Module dedicated to [Function] parsing.

use core::slice::Windows;

use super::word::{Kind, Word};
use super::{PositionnedString, Value};
use crate::ScriptError;

/// Structure describing a textual requirement.
///
/// It owns the requirement name.
#[derive(Clone, Debug, Default)]
pub struct Function {
    pub name: PositionnedString,
    pub parameters: Vec<Value>,
}

impl Function {
    pub fn build_from_parameters(
        name: PositionnedString,
        mut iter: &mut Windows<Word>,
    ) -> Result<Self, ScriptError> {
        let mut parameters = Vec::new();

        let possible_closing_parenthesis;
        match iter.next().map(|s| (&s[0], &s[1])) {
            Some((w, nw)) if w.kind != Some(Kind::OpeningParenthesis) => {
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
            Some(w) if w.kind == Some(Kind::ClosingParenthesis) => {}
            _ => loop {
                parameters.push(Value::build_from_first_item(&mut iter)?);

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

        Ok(Self { name, parameters })
    }
}
