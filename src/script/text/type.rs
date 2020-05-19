
use crate::script::error::ScriptError;

use super::word::{expect_word_kind, Kind, Word};

pub struct Type {
    pub structure: Option<String>,
    pub name: String
}

impl Type {
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
