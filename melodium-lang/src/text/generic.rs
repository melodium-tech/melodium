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
            .ok_or_else(|| ScriptError::end_of_script(53))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(54, w.clone(), &[Kind::Name]))
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
        iter: &mut Windows<Word>,
        following_word: Option<&Word>,
    ) -> Result<Self, ScriptError> {
        let (mut r#type, _) = Type::build_from_next(None, name, iter, following_word)?;

        Ok(Self {
            annotations: annotations.or(r#type.annotations.take()),
            r#type,
        })
    }
}
