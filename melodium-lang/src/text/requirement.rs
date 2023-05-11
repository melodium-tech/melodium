//! Module dedicated to [Requirement] parsing.

use core::slice::Windows;

use super::word::{Kind, Word};
use super::PositionnedString;
use crate::ScriptError;

/// Structure describing a textual requirement.
///
/// It owns the requirement name.
#[derive(Clone, Debug)]
pub struct Requirement {
    pub name: PositionnedString,
}

impl Requirement {
    /// Build requirement by parsing words.
    ///
    /// * `iter`: Iterator over words list, next() being expected to be the context required, see [Kind::Context].
    ///
    pub fn build(iter: &mut Windows<Word>) -> Result<Self, ScriptError> {
        let name = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(53))
            .and_then(|w| {
                if w.kind != Some(Kind::Context) {
                    Err(ScriptError::word(54, w.clone(), &[Kind::Context]))
                } else {
                    Ok(w.into())
                }
            })?;

        Ok(Self { name })
    }
}
