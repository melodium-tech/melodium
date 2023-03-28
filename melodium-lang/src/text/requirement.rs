//! Module dedicated to [Requirement] parsing.

use super::word::{expect_word_kind, Kind, Word};
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
    /// ```
    /// # use melodium_lang::ScriptError;
    /// # use melodium_lang::text::word::*;
    /// # use melodium_lang::text::requirement::Requirement;
    /// let words = get_words("require @Signal").unwrap();
    /// let mut iter = words.iter();
    ///
    /// let require_keyword = expect_word_kind(Kind::Name, "Keyword expected.", &mut iter)?;
    /// assert_eq!(require_keyword.string, "require");
    ///
    /// let requirement = Requirement::build(&mut iter)?;
    ///
    /// assert_eq!(requirement.name.string, "@Signal");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {
        let name = expect_word_kind(Kind::Context, "Context name expected.", &mut iter)?;

        Ok(Self { name })
    }
}
