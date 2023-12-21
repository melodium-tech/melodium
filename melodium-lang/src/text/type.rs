//! Module dedicated to [Type] parsing.

use super::word::{Kind, Word};
use super::{CommentsAnnotations, PositionnedString};
use crate::ScriptError;
use core::slice::Windows;
use std::collections::HashMap;

/// Structure describing a textual type.
///
/// It owns a name, and a flow or structure, if any.
#[derive(Clone, Debug)]
pub struct Type {
    pub annotations: Option<CommentsAnnotations>,
    pub level_structure: Vec<PositionnedString>,
    pub name: PositionnedString,
}

impl Type {
    /// Build a type by parsing words.
    ///
    /// * `iter`: Iterator over words list, next() being expected to be either the name or structure.
    ///
    pub fn build(
        iter: &mut Windows<Word>,
        global_annotations: &mut HashMap<Word, CommentsAnnotations>,
    ) -> Result<(Self, Word), ScriptError> {
        let step_type = iter.next();
        let (annotations, first_name_or_structure) = step_type
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(21))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(22, w.clone(), &[Kind::Name]))
                } else {
                    Ok((global_annotations.remove(w), w.into()))
                }
            })?;

        Self::build_from_next(
            annotations,
            first_name_or_structure,
            iter,
            step_type.map(|s| &s[1]),
        )
    }

    /// Build a type by parsing words, considering name already been parsed.
    ///
    /// * `iter`: Iterator over words list.
    ///
    pub fn build_from_next(
        annotations: Option<CommentsAnnotations>,
        first_name_or_structure: PositionnedString,
        iter: &mut Windows<Word>,
        following_word: Option<&Word>,
    ) -> Result<(Self, Word), ScriptError> {
        let mut following_word = following_word.map(|w| w.clone());
        let mut next_word = None;
        let mut structure = vec![first_name_or_structure];
        let mut open_chevrons: usize = 0;
        loop {
            match following_word {
                Some(w) if w.kind == Some(Kind::OpeningChevron) => {
                    iter.next(); // Skipping chevron
                    open_chevrons += 1;

                    let sub_step = iter.next();

                    let deeper_name_or_structure = sub_step
                        .map(|s| &s[0])
                        .ok_or_else(|| ScriptError::end_of_script(23))
                        .and_then(|w| {
                            if w.kind != Some(Kind::Name) {
                                Err(ScriptError::word(24, w.clone(), &[Kind::Name]))
                            } else {
                                Ok(w.into())
                            }
                        })?;

                    structure.push(deeper_name_or_structure);

                    following_word = sub_step.map(|s| s[1].clone());
                }
                Some(w) if open_chevrons > 0 && w.kind == Some(Kind::ClosingChevron) => break,
                Some(w) if open_chevrons == 0 => {
                    next_word = Some(w.clone());
                    break;
                }
                Some(w) => {
                    return Err(ScriptError::word(
                        28,
                        w.clone(),
                        &[Kind::OpeningChevron, Kind::ClosingChevron],
                    ))
                }
                None => return Err(ScriptError::end_of_script(27)),
            }
        }

        for _ in 0..open_chevrons {
            match iter.next().map(|s| (&s[0], &s[1])) {
                Some((w, nw)) if w.kind == Some(Kind::ClosingChevron) => {
                    next_word = Some(nw.clone())
                }
                Some((w, _)) => {
                    return Err(ScriptError::word(29, w.clone(), &[Kind::ClosingChevron]))
                }
                None => return Err(ScriptError::end_of_script(30)),
            }
        }

        Ok((
            Self {
                annotations,
                name: structure.pop().unwrap(),
                level_structure: structure,
            },
            next_word.unwrap(),
        ))
    }
}

#[cfg(test)]
mod tests {

    use super::super::word::*;
    use super::*;

    #[test]
    fn test_well_catching_name_alone() {
        let text = "Int";
        let mut words = get_words(text).unwrap();
        words.push(Word::default());
        let mut iter = words.windows(2);

        let r#type = Type::build(&mut iter, &mut HashMap::new()).unwrap().0;

        assert!(r#type.level_structure.is_empty());
        assert_eq!(r#type.name.string, "Int");
    }

    #[test]
    fn test_well_catching_first_level_and_name() {
        let text = "Vec<Int>";
        let mut words = get_words(text).unwrap();
        words.push(Word::default());
        let mut iter = words.windows(2);

        let r#type = Type::build(&mut iter, &mut HashMap::new()).unwrap().0;

        assert_eq!(r#type.level_structure.first().unwrap().string, "Vec");
        assert_eq!(r#type.name.string, "Int");
    }

    #[test]
    fn test_well_catching_first_and_second_level_and_name() {
        let text = "Stream<Vec<Int>>";
        let mut words = get_words(text).unwrap();
        words.push(Word::default());
        let mut iter = words.windows(2);

        let r#type = Type::build(&mut iter, &mut HashMap::new()).unwrap().0;

        assert_eq!(r#type.level_structure[0].string, "Stream");
        assert_eq!(r#type.level_structure[1].string, "Vec");
        assert_eq!(r#type.name.string, "Int");
    }
}
