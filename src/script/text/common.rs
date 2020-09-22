
//! Contains convenience functions and tools for text parsing.

use crate::script::error::ScriptError;
use super::PositionnedString;
use super::word::{expect_word, expect_word_kind, Kind, Word};
use super::parameter::Parameter;

/// Build a parameter list by parsing words.
/// 
/// * `iter`: Iterator over words list, next() being expected to be the opening parenthesis.
/// ```
/// # use melodium_rust::script::error::ScriptError;
/// # use melodium_rust::script::text::word::*;
/// # use melodium_rust::script::text::common::parse_parameters;
/// 
/// let text = r##"
/// (path: Vec<String>, sampleRate: Int = 44100, frameSize: Int = 4096, hopSize: Int = 2048, windowingType: String)
/// "##;
/// 
/// let words = get_words(text).unwrap();
/// let mut iter = words.iter();
/// 
/// let parameters = parse_parameters(&mut iter)?;
/// 
/// assert_eq!(parameters.len(), 5);
/// # Ok::<(), ScriptError>(())
/// ```
pub fn parse_parameters(mut iter: &mut std::slice::Iter<Word>) -> Result<Vec<Parameter>, ScriptError> {

    expect_word_kind(Kind::OpeningParenthesis, "Parameters declaration expected '('.", &mut iter)?;

    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {

        let word = expect_word("Unexpected end of script.", &mut iter)?;

        if first_param && word.kind == Some(Kind::ClosingParenthesis) {
            break;
        }
        else if word.kind == Some(Kind::Name) {
            first_param = false;

            expect_word_kind(Kind::Colon, "Parameter type declaration expected.", &mut iter)?;
            parameters.push(Parameter::build_from_type(PositionnedString{string: word.text, position: word.position}, &mut iter)?);

            let delimiter = expect_word("Unexpected end of script.", &mut iter)?;
            
            if delimiter.kind == Some(Kind::Comma) {
                continue;
            }
            else if delimiter.kind == Some(Kind::ClosingParenthesis) {
                break;
            }
            else {
                return Err(ScriptError::word("Comma or closing parenthesis expected.".to_string(), delimiter.text, delimiter.position));
            }
        }
        else {
            return Err(ScriptError::word("Parameter declaration expected.".to_string(), word.text, word.position));
        }
    }

    Ok(parameters)
}

/// Build a model list by parsing words.
/// 
/// * `iter`: Iterator over words list, next() being expected to be the opening parenthesis.
/// ```
/// # use melodium_rust::script::error::ScriptError;
/// # use melodium_rust::script::text::word::*;
/// # use melodium_rust::script::text::common::parse_parametric_models;
/// 
/// let text = r##"
/// [Files: FileManager, Audio: AudioManager]
/// "##;
/// 
/// let words = get_words(text).unwrap();
/// let mut iter = words.iter();
/// 
/// let models = parse_parametric_models(&mut iter)?;
/// 
/// assert_eq!(models.len(), 2);
/// # Ok::<(), ScriptError>(())
/// ```
pub fn parse_parametric_models(mut iter: &mut std::slice::Iter<Word>) -> Result<Vec<Parameter>, ScriptError> {

    expect_word_kind(Kind::OpeningBracket, "Models declaration expected '['.", &mut iter)?;

    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {

        let word = expect_word("Unexpected end of script.", &mut iter)?;

        if first_param && word.kind == Some(Kind::ClosingBracket) {
            break;
        }
        else if word.kind == Some(Kind::Name) {
            first_param = false;

            expect_word_kind(Kind::Colon, "Model type declaration expected.", &mut iter)?;
            parameters.push(Parameter::build_from_type(PositionnedString{string: word.text, position: word.position}, &mut iter)?);

            let delimiter = expect_word("Unexpected end of script.", &mut iter)?;
            
            if delimiter.kind == Some(Kind::Comma) {
                continue;
            }
            else if delimiter.kind == Some(Kind::ClosingParenthesis) {
                break;
            }
            else {
                return Err(ScriptError::word("Comma or closing bracket expected.".to_string(), delimiter.text, delimiter.position));
            }
        }
        else {
            return Err(ScriptError::word("Model declaration expected.".to_string(), word.text, word.position));
        }
    }

    Ok(parameters)
}

