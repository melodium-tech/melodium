//! Module dedicated to [Type] parsing.

use core::slice::Windows;

use super::word::{Kind, Word};
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
    pub fn build(iter: &mut Windows<Word>) -> Result<(Self, Word), ScriptError> {
        let step_type = iter.next();
        let first_name_or_structure = step_type
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(21))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(22, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.into())
                }
            })?;

        match step_type.map(|s| &s[1]) {
            Some(w) if w.kind == Some(Kind::OpeningChevron) => {
                iter.next(); // Skipping chevron

                let sub_step = iter.next();

                let second_name_or_structure = sub_step
                    .map(|s| &s[0])
                    .ok_or_else(|| ScriptError::end_of_script(23))
                    .and_then(|w| {
                        if w.kind != Some(Kind::Name) {
                            Err(ScriptError::word(24, w.clone(), &[Kind::Name]))
                        } else {
                            Ok(w.into())
                        }
                    })?;

                match sub_step.map(|s| &s[1]) {
                    Some(w) if w.kind == Some(Kind::OpeningChevron) => {
                        iter.next(); // Skipping chevron

                        let name = iter
                            .next()
                            .map(|s| &s[0])
                            .ok_or_else(|| ScriptError::end_of_script(27))
                            .and_then(|w| {
                                if w.kind != Some(Kind::Name) {
                                    Err(ScriptError::word(28, w.clone(), &[Kind::Name]))
                                } else {
                                    Ok(w.into())
                                }
                            })?;

                        let mut next_word = None;
                        for _ in 0..2 {
                            iter.next()
                                .ok_or_else(|| ScriptError::end_of_script(29))
                                .map(|s| (&s[0], &s[1]))
                                .and_then(|(w, nw)| {
                                    if w.kind != Some(Kind::ClosingChevron) {
                                        Err(ScriptError::word(
                                            30,
                                            w.clone(),
                                            &[Kind::ClosingChevron],
                                        ))
                                    } else {
                                        next_word = Some(nw.clone());
                                        Ok(())
                                    }
                                })?;
                        }

                        Ok((
                            Self {
                                first_level_structure: Some(first_name_or_structure),
                                second_level_structure: Some(second_name_or_structure),
                                name,
                            },
                            next_word.unwrap(),
                        ))
                    }
                    _ => {
                        let mut next_word = None;
                        iter.next()
                            .ok_or_else(|| ScriptError::end_of_script(25))
                            .map(|s| (&s[0], &s[1]))
                            .and_then(|(w, nw)| {
                                if w.kind != Some(Kind::ClosingChevron) {
                                    Err(ScriptError::word(26, w.clone(), &[Kind::ClosingChevron]))
                                } else {
                                    next_word = Some(nw.clone());
                                    Ok(())
                                }
                            })?;
                        Ok((
                            Self {
                                first_level_structure: Some(first_name_or_structure),
                                second_level_structure: None,
                                name: second_name_or_structure,
                            },
                            next_word.unwrap(),
                        ))
                    }
                }
            }
            Some(w) => Ok((
                Self {
                    first_level_structure: None,
                    second_level_structure: None,
                    name: first_name_or_structure,
                },
                w.clone(),
            )),
            None => return Err(ScriptError::end_of_script(61)),
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
        let mut words = get_words(text).unwrap();
        words.push(Word::default());
        let mut iter = words.windows(2);

        let r#type = Type::build(&mut iter).unwrap().0;

        assert!(r#type.first_level_structure.is_none());
        assert!(r#type.second_level_structure.is_none());
        assert_eq!(r#type.name.string, "Int");
    }

    #[test]
    fn test_well_catching_first_level_and_name() {
        let text = "Vec<Int>";
        let mut words = get_words(text).unwrap();
        words.push(Word::default());
        let mut iter = words.windows(2);

        let r#type = Type::build(&mut iter).unwrap().0;

        assert_eq!(r#type.first_level_structure.unwrap().string, "Vec");
        assert!(r#type.second_level_structure.is_none());
        assert_eq!(r#type.name.string, "Int");
    }

    #[test]
    fn test_well_catching_first_and_second_level_and_name() {
        let text = "Stream<Vec<Int>>";
        let mut words = get_words(text).unwrap();
        words.push(Word::default());
        let mut iter = words.windows(2);

        let r#type = Type::build(&mut iter).unwrap().0;

        assert_eq!(r#type.first_level_structure.unwrap().string, "Stream");
        assert_eq!(r#type.second_level_structure.unwrap().string, "Vec");
        assert_eq!(r#type.name.string, "Int");
    }
}
