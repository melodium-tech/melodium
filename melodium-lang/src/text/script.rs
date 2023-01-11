
//! Module dedicated to [Script](struct.Script.html) parsing.

use std::collections::HashMap;

use crate::ScriptError;

use super::PositionnedString;
use super::word::{expect_word, get_words, Kind, Position};
use super::r#use::Use;
use super::annotation::Annotation;
use super::model::Model;
use super::sequence::Sequence;

/// Structure managing and describing textual script.
/// 
/// It owns the whole script text, as well as parsed attributes, including [Uses](../use/struct.Use.html), [Annotations](../annotation/struct.Annotation.html), [Models](../model/struct.Model.html), and [Sequences](../sequence/struct.Sequence.html).
/// There is no logical coherence involved there, only syntax analysis and parsing.
#[derive(Clone, Debug)]
pub struct Script {
    pub text: String,
    pub uses: Vec<Use>,
    pub annotations: Vec<Annotation>,
    pub models: Vec<Model>,
    pub sequences: Vec<Sequence>,
}

impl Script {
    /// Build script by parsing the whole content.
    /// 
    /// This is the main function of the whole [text module](../index.html), it process the entire textual content of script and build a syntax tree.
    /// It also makes a copy of `text` and keeps it by its own.
    /// 
    /// * `text`: The text of the script itself.
    /// 
    /// # Note
    /// It doesn't check any logic, only syntax analysis and parsing.
    /// 
    /// ```
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script;
    /// 
    /// let text = r##"
    /// use project/subpath/to/utils::MakeHPCP
    /// 
    /// // Main sequence
    /// sequence Main()
	///     origin PrepareAudioFiles(path="Musique/", sampleRate=44100, frameSize=4096, hopSize=2048, windowingType="blackmanharris92")
    ///     require @File
    ///     require @Signal
    /// {
    /// 
    ///     MakeHPCP(sampleRate=@Signal[sampleRate], minFrequency=40, maxFrequency=5000, harmonics=8, size=120)
    /// 
    ///     PrepareAudioFiles.spectrum -> MakeHPCP.spectrum
    /// }
    /// "##;
    /// 
    /// let script = Script::build(text)?;
    /// 
    /// assert_eq!(script.sequences.len(), 1);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(text: & str) -> Result<Self, ScriptError> {
        let mut uses = Vec::new();
        let mut annotations = Vec::new();
        let mut models = Vec::new();
        let mut sequences = Vec::new();

        let words = get_words(text);
        if words.is_err() {
            let err_words = words.unwrap_err();
            let err_word = err_words.last();
            if err_word.is_some() {
                let err_word = err_word.unwrap();
                return Err(ScriptError::word("Unkown word.".to_string(), err_word.text.to_string(), err_word.position));
            }
            else {
                return Err(ScriptError::end_of_script("Script is empty.".to_string()));
            }
        }

        let mut words = words.unwrap();

        // Documentation purpose: associating every documentation item
        // with the nearest next word that is not a comment.
        let mut documented_items = HashMap::new();
        let mut last_doc = None;
        for word in words.iter() {
            if word.kind == Some(Kind::Comment) {
                if word.text.starts_with("///") {
                    last_doc = Some(PositionnedString {
                        string: word.text.strip_prefix("///").unwrap().to_string(),
                        position: Position {
                            absolute_position: word.position.absolute_position + 3,
                            line_number: word.position.line_number,
                            line_position: word.position.line_position + 3,
                        }
                    });
                }
                else if word.text.starts_with("/**") {
                    last_doc = Some(PositionnedString {
                        string: word.text.strip_prefix("/**").unwrap().strip_suffix("*/").unwrap().to_string(),
                        position: Position {
                            absolute_position: word.position.absolute_position + 3,
                            line_number: word.position.line_number,
                            line_position: word.position.line_position + 3,
                        }
                    });
                }
            }
            else if word.kind == Some(Kind::Annotation) {
                continue;
            }
            else if let Some(doc) = last_doc {
                documented_items.insert(word.clone(), doc);
                last_doc = None;
            }
        }

        // Removing all comments.
        words.retain(|w| w.kind != Some(Kind::Comment));
        let words = words;

        let mut iter = words.iter();
        loop {
            let possible_word = expect_word("Reached end of script.", &mut iter);
            if possible_word.is_ok() {
                let word = possible_word.unwrap();

                if word.kind == Some(Kind::Annotation) {
                    annotations.push(Annotation{text: PositionnedString{string: word.text, position: word.position}});
                }
                else if word.kind == Some(Kind::Name) {
                    if word.text == "use" {
                        uses.push(Use::build(&mut iter)?);
                    }
                    else if word.text == "model" {
                        models.push(Model::build(&mut iter, documented_items.remove(&word))?);
                    }
                    else if word.text == "sequence" {
                        sequences.push(Sequence::build(&mut iter, documented_items.remove(&word))?);
                    }
                    else {
                        return Err(ScriptError::word("Unkown declaration.".to_string(), word.text, word.position));
                    }
                }
                else {
                    return Err(ScriptError::word("Unexpected symbol.".to_string(), word.text, word.position));
                }
            }
            else {
                // Not an error, just reached end of script.
                break;
            }
        }

        Ok(Self{
            text: text.to_string(),
            uses,
            annotations,
            models,
            sequences,
        })
    }
}
