//! Module dedicated to [Generic] parsing.

use super::word::*;
use super::CommentsAnnotations;
use super::PositionnedString;
use crate::ScriptError;
use core::slice::Windows;
use std::collections::HashMap;

/// Structure describing a generic.
///
/// It owns a name, describing either the generic name itself, or the designated type.
#[derive(Clone, Debug)]
pub struct Generic {
    pub annotations: Option<CommentsAnnotations>,
    pub name: PositionnedString,
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
        let (annotations, name) = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(53))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(54, w.clone(), &[Kind::Name]))
                } else {
                    Ok((global_annotations.remove(&w), w.into()))
                }
            })?;

        Ok(Self { annotations, name })
    }
}
