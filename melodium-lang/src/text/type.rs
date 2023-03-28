//! Module dedicated to [Type] parsing.

use super::word::{expect_word_kind, Kind, Word};
use super::PositionnedString;
use crate::ScriptError;

/// Structure describing a textual type.
///
/// It owns a name, and a flow or structure, if any.
#[derive(Clone, Debug)]
pub struct Type {
    pub first_level_structure: Option<PositionnedString>,
    pub second_level_structure: Option<PositionnedString>,
    pub name: PositionnedString,
}

impl Type {
    /// Build a type by parsing words.
    ///
    /// * `iter`: Iterator over words list, next() being expected to be either the name or structure.
    ///
    /// ```
    /// # use melodium_lang::ScriptError;
    /// # use melodium_lang::text::word::*;
    /// # use melodium_lang::text::r#type::Type;
    /// let text = "Vec<Int>";
    ///
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    ///
    /// let r#type = Type::build(&mut iter)?;
    ///
    /// assert_eq!(r#type.name.string, "Int");
    /// assert_eq!(r#type.first_level_structure.unwrap().string, "Vec");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {
        let first_name_or_structure =
            expect_word_kind(Kind::Name, "Type name expected.", &mut iter)?;

        // We _clone_ the iterator (in case next word doesn't rely on Type) and doesn't make our expectation to fail if not satisfied.
        let possible_opening_chevron =
            expect_word_kind(Kind::OpeningChevron, "", &mut iter.clone());
        // In that case, we are expecting a name or structure.
        if possible_opening_chevron.is_ok() {
            // We discard the opening chevron.
            iter.next();
            let second_name_or_structure =
                expect_word_kind(Kind::Name, "Type name expected.", &mut iter)?;

            // We _clone_ the iterator (in case next word doesn't rely on Type) and doesn't make our expectation to fail if not satisfied.
            let possible_opening_chevron =
                expect_word_kind(Kind::OpeningChevron, "", &mut iter.clone());
            // In that case, we are really expecting a name.
            if possible_opening_chevron.is_ok() {
                // We discard the opening chevron.
                iter.next();
                let name = expect_word_kind(Kind::Name, "Type name expected.", &mut iter)?;

                for _ in 0..2 {
                    expect_word_kind(Kind::ClosingChevron, "Closing chevron expected.", &mut iter)?;
                }

                Ok(Self {
                    first_level_structure: Some(first_name_or_structure),
                    second_level_structure: Some(second_name_or_structure),
                    name,
                })
            } else {
                expect_word_kind(Kind::ClosingChevron, "Closing chevron expected.", &mut iter)?;

                Ok(Self {
                    first_level_structure: Some(first_name_or_structure),
                    second_level_structure: None,
                    name: second_name_or_structure,
                })
            }
        } else {
            Ok(Self {
                first_level_structure: None,
                second_level_structure: None,
                name: first_name_or_structure,
            })
        }
    }
}

#[cfg(test)]
mod tests {

    use super::super::word::*;
    use super::*;

    #[test]
    fn test_well_catching_name_alone() {
        let text = "Int";
        let words = get_words(text).unwrap();
        let mut iter = words.iter();

        let r#type = Type::build(&mut iter).unwrap();

        assert!(r#type.first_level_structure.is_none());
        assert!(r#type.second_level_structure.is_none());
        assert_eq!(r#type.name.string, "Int");
    }

    #[test]
    fn test_well_catching_first_level_and_name() {
        let text = "Vec<Int>";
        let words = get_words(text).unwrap();
        let mut iter = words.iter();

        let r#type = Type::build(&mut iter).unwrap();

        assert_eq!(r#type.first_level_structure.unwrap().string, "Vec");
        assert!(r#type.second_level_structure.is_none());
        assert_eq!(r#type.name.string, "Int");
    }

    #[test]
    fn test_well_catching_first_and_second_level_and_name() {
        let text = "Stream<Vec<Int>>";
        let words = get_words(text).unwrap();
        let mut iter = words.iter();

        let r#type = Type::build(&mut iter).unwrap();

        assert_eq!(r#type.first_level_structure.unwrap().string, "Stream");
        assert_eq!(r#type.second_level_structure.unwrap().string, "Vec");
        assert_eq!(r#type.name.string, "Int");
    }
}
