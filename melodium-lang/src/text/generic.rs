//! Module dedicated to [Generic] parsing.

use super::word::*;
use super::CommentsAnnotations;
use super::Type;
use crate::ScriptError;
use core::slice::Windows;
use std::collections::HashMap;

/// Structure describing a generic.
///
/// It owns a name, describing either the generic name itself, or the designated type.
#[derive(Clone, Debug)]
pub struct Generic {
    pub annotations: Option<CommentsAnnotations>,
    pub r#type: Type,
    pub traits: Vec<PositionnedString>,
}

impl Generic {
    /// Build generic by parsing words.
    ///
    /// * `iter`: Iterator over words list, next() being expected to be the generic name.
    ///
    pub fn build(
        iter: &mut Windows<Word>,
        global_annotations: &mut HashMap<Word, CommentsAnnotations>,
    ) -> Result<Self, ScriptError> {
        let step_type = iter.next();
        let (annotations, name) = step_type
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(169))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(170, w.clone(), &[Kind::Name]))
                } else {
                    Ok((global_annotations.remove(&w), w.into()))
                }
            })?;

        Generic::build_from_next(annotations, name, iter, step_type.map(|s| &s[1]))
    }

    /// Build a generic by parsing words, considering name already been parsed.
    ///
    /// * `iter`: Iterator over words list.
    ///
    pub fn build_from_next(
        annotations: Option<CommentsAnnotations>,
        name: PositionnedString,
        mut iter: &mut Windows<Word>,
        following_word: Option<&Word>,
    ) -> Result<Self, ScriptError> {
        let (mut r#type, next_word) = Type::build_from_next(None, name, iter, following_word)?;

        let traits = match next_word.kind {
            Some(Kind::Colon) => {
                // Skip ':'
                iter.next();
                Self::parse_traits(&mut iter)?
            }
            _ => Vec::new(),
        };

        Ok(Self {
            annotations: annotations.or(r#type.annotations.take()),
            r#type,
            traits,
        })
    }

    fn parse_traits(iter: &mut Windows<Word>) -> Result<Vec<PositionnedString>, ScriptError> {
        let mut traits = Vec::new();
        while let Some((trait_name, nw)) = iter.next().map(|s| (&s[0], &s[1])) {
            if trait_name.kind == Some(Kind::Name) {
                traits.push(trait_name.into());

                match nw.kind {
                    Some(Kind::Plus) => continue,
                    None => return Err(ScriptError::end_of_script(179)),
                    _ => break,
                }
            } else {
                return Err(ScriptError::word(176, trait_name.clone(), &[Kind::Name]));
            }
        }

        Ok(traits)
    }
}
