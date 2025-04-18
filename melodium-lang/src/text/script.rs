//! Module dedicated to [Script] parsing.

use super::annotation::Annotation;
use super::model::Model;
use super::r#use::Use;
use super::treatment::Treatment;
use super::word::{get_words, Kind, Position, Word};
use super::{CommentsAnnotations, PositionnedString};
use crate::ScriptError;
use std::collections::HashMap;

/// Structure managing and describing textual script.
///
/// It owns the whole script text, as well as parsed attributes, including [Use]s, [Annotation]s, [Model]s, and [Treatment]s.
/// There is no logical coherence involved there, only syntax analysis and parsing.
#[derive(Clone, Debug)]
pub struct Script {
    pub text: String,
    pub uses: Vec<Use>,
    pub annotations: Vec<Annotation>,
    pub models: Vec<Model>,
    pub treatments: Vec<Treatment>,
}

impl Script {
    /// Build script by parsing the whole content.
    ///
    /// This is the main function of the whole [text module](super), it process the entire textual content of script and build a syntax tree.
    /// It also makes a copy of `text` and keeps it by its own.
    ///
    /// * `text`: The text of the script itself.
    ///
    /// # Note
    /// It doesn't check any logic, only syntax analysis and parsing.
    ///
    pub fn build(text: &str) -> Result<Self, ScriptError> {
        let mut uses = Vec::new();
        let mut models = Vec::new();
        let mut treatments = Vec::new();

        let words = get_words(text);
        if let Err(err_words) = words {
            if let Some(err_word) = err_words.last() {
                return Err(ScriptError::word(19, err_word.clone(), &[]));
            } else {
                return Err(ScriptError::end_of_script(20));
            }
        }

        let mut words = words.unwrap();

        // Annotation and documentation purpose: associating every annotation, attribute, comment, or documentation item
        // with the nearest next word that is something else than one of those.
        let mut annotated_items = HashMap::new();
        let mut last_doc = None;
        let mut annotations = Vec::new();
        let mut comments = Vec::new();
        for word in words.iter() {
            if word.kind == Some(Kind::Comment) {
                if word.text.starts_with("///") {
                    last_doc = Some(PositionnedString {
                        string: word.text.strip_prefix("///").unwrap().to_string(),
                        position: Position {
                            absolute_position: word.position.absolute_position + 3,
                            line_number: word.position.line_number,
                            line_position: word.position.line_position + 3,
                        },
                    });
                } else if word.text.starts_with("/**") {
                    last_doc = Some(PositionnedString {
                        string: word
                            .text
                            .strip_prefix("/**")
                            .unwrap()
                            .strip_suffix("*/")
                            .unwrap()
                            .to_string(),
                        position: Position {
                            absolute_position: word.position.absolute_position + 3,
                            line_number: word.position.line_number,
                            line_position: word.position.line_position + 3,
                        },
                    });
                } else if word.text.starts_with("//") {
                    comments.push(PositionnedString {
                        string: word.text.strip_prefix("//").unwrap().to_string(),
                        position: Position {
                            absolute_position: word.position.absolute_position + 2,
                            line_number: word.position.line_number,
                            line_position: word.position.line_position + 2,
                        },
                    });
                } else if word.text.starts_with("/*") {
                    comments.push(PositionnedString {
                        string: word
                            .text
                            .strip_prefix("/*")
                            .unwrap()
                            .strip_suffix("*/")
                            .unwrap()
                            .to_string(),
                        position: Position {
                            absolute_position: word.position.absolute_position + 2,
                            line_number: word.position.line_number,
                            line_position: word.position.line_position + 2,
                        },
                    });
                }
            } else if word.kind == Some(Kind::Annotation) {
                annotations.push(Annotation { text: word.into() });
            } else {
                annotated_items.insert(
                    word.clone(),
                    CommentsAnnotations {
                        doc: last_doc,
                        comments,
                        annotations,
                    },
                );
                last_doc = None;
                comments = Vec::new();
                annotations = Vec::new();
            }
        }

        // Removing all comments and annotations.
        words.retain(|w| w.kind != Some(Kind::Comment) && w.kind != Some(Kind::Annotation));

        // Adding a last word for eof signal
        words.push(Word::default());

        let words = words;

        let mut iter = words.windows(2);
        loop {
            match iter.next().map(|s| &s[0]) {
                Some(w) if w.kind == Some(Kind::Name) => match w.text.as_str() {
                    "use" => uses.push(Use::build(&mut iter)?),
                    "model" => models.push(Model::build(
                        &mut iter,
                        annotated_items.remove(&w),
                        &mut annotated_items,
                    )?),
                    "treatment" => treatments.push(Treatment::build(
                        &mut iter,
                        annotated_items.remove(&w),
                        &mut annotated_items,
                    )?),
                    _ => {
                        return Err(ScriptError::declaration_expected(
                            51,
                            w.clone(),
                            &["use", "model", "treatment"],
                        ))
                    }
                },
                Some(w) => {
                    return Err(ScriptError::word(
                        52,
                        w.clone(),
                        &[Kind::Annotation, Kind::Name],
                    ))
                }
                None => break,
            }
        }

        Ok(Self {
            text: text.to_string(),
            uses,
            annotations,
            models,
            treatments,
        })
    }
}
