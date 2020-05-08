
use crate::script::error::ScriptError;

use super::word::{expect_word, get_words, Kind};
use super::annotation::Annotation;
use super::model::Model;
use super::sequence::Sequence;

pub struct Script {
    pub text: String,
    pub annotations: Vec<Annotation>,
    pub models: Vec<Model>,
    pub sequences: Vec<Sequence>,
}

impl Script {
    pub fn build(text: & String) -> Result<Self, ScriptError> {
        let mut annotations = Vec::new();
        let mut models = Vec::new();
        let mut sequences = Vec::new();

        let words = get_words(text);
        if words.is_err() {
            let err_words = words.unwrap_err();
            let err_word = err_words.last();
            if err_word.is_some() {
                let err_word = err_word.unwrap();
                return Err(ScriptError::new("Unkown word.".to_string(), err_word.text.to_string(), err_word.line, err_word.line_position, err_word.absolute_position));
            }
            else {
                return Err(ScriptError::end_of_script("Script is empty.".to_string()));
            }
        }

        let mut words = words.unwrap();

        // Removing all comments.
        words.retain(|w| w.kind != Some(Kind::Comment));
        let words = words;

        let mut iter = words.iter();
        loop {
            let possible_word = expect_word("Reached end of script.", &mut iter);
            if possible_word.is_ok() {
                let word = possible_word.unwrap();

                if word.kind == Some(Kind::Annotation) {
                    annotations.push(Annotation{text: word.text});
                }
                else if word.kind == Some(Kind::Name) {
                    if word.text == "model" {
                        models.push(Model::build(&mut iter)?);
                    }
                    else if word.text == "sequence" {
                        sequences.push(Sequence::build(&mut iter)?);
                    }
                    else {
                        return Err(ScriptError::new("Unkown declaration.".to_string(), word.text, word.line, word.line_position, word.absolute_position));
                    }
                }
                else {
                    return Err(ScriptError::new("Unexpected symbol.".to_string(), word.text, word.line, word.line_position, word.absolute_position));
                }
            }
            else {
                // Not an error, just reached end of script.
                break;
            }
        }

        Ok(Self{
            text: text.to_string(),
            annotations,
            models,
            sequences,
        })
    }
}
