
//! Module dedicated to [Type](struct.Type.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::{expect_word_kind, Kind, Word};

/// Structure describing a textual type.
/// 
/// It owns a name, and a structure, if any.
#[derive(Clone)]
pub struct Type {
    pub structure: Option<PositionnedString>,
    pub name: PositionnedString
}

impl Type {
    /// Build a type by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be either the name or structure.
    /// 
    /// ```
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::word::*;
    /// # use melodium_rust::script::text::r#type::Type;
    /// let text = "Vec<Int>";
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let r#type = Type::build(&mut iter)?;
    /// 
    /// assert_eq!(r#type.name, "Int");
    /// assert_eq!(r#type.structure, Some("Vec".to_string()));
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name_or_structure = expect_word_kind(Kind::Name, "Type name expected.", &mut iter)?;

        // We _clone_ the iterator (in case next word doesn't rely on Type) and doesn't make our expectation to fail if not satisfied.
        let possible_opening_chevron = expect_word_kind(Kind::OpeningChevron, "", &mut iter.clone());
        // In that case, we are really expecting a name and structure.
        if possible_opening_chevron.is_ok() {
            // We discard the opening chevron.
            iter.next();
            let name = expect_word_kind(Kind::Name, "Type name expected.", &mut iter)?;
            expect_word_kind(Kind::ClosingChevron, "Closing chevron expected.", &mut iter)?;

            Ok(Self {
                structure: Some(name_or_structure),
                name,
            })
        }
        else {
            Ok(Self {
                name: name_or_structure,
                structure: None
            })
        }
    }
}
