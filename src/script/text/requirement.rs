
//! Module dedicated to [Requirement](struct.Requirement.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::{expect_word_kind, Kind, Word};

/// Structure describing a textual requirement.
/// 
/// It owns the requirement name.
#[derive(Clone)]
pub struct Requirement {
    pub name: PositionnedString
}

impl Requirement {
    /// Build requirement by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the named reference required, see [Kind::Reference](../word/enum.Kind.html#variant.Reference).
    /// 
    /// ```
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::word::*;
    /// # use melodium_rust::script::text::requirement::Requirement;
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

        let name = expect_word_kind(Kind::Reference, "Requirement name expected.", &mut iter)?;

        Ok(Self {
            name
        })
    }
}
