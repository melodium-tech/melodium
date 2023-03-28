//! Contains convenience functions and tools for text parsing.

use super::parameter::Parameter;
use super::word::{expect_word, expect_word_kind, Kind, Word};
use super::PositionnedString;
use crate::ScriptError;

/// Build a parameter declaration list by parsing words.
///
/// * `iter`: Iterator over words list, next() being expected to be the first parameter, _not_ parenthesis.
/// ```
/// # use melodium_lang::ScriptError;
/// # use melodium_lang::text::word::*;
/// # use melodium_lang::text::common::parse_parameters_declarations;
///
/// let text = r##"
/// (path: Vec<String>, const sampleRate: u32 = 44100, const frameSize: u32 = 4096, const hopSize: u32 = 2048, var windowingType: string)
/// "##;
///
/// let words = get_words(text).unwrap();
/// let mut iter = words.iter();
///
/// expect_word_kind(Kind::OpeningParenthesis, "Parameters declaration expected '('.", &mut iter)?;
/// let parameters = parse_parameters_declarations(&mut iter)?;
///
/// assert_eq!(parameters.len(), 5);
/// # Ok::<(), ScriptError>(())
/// ```
pub fn parse_parameters_declarations(
    mut iter: &mut std::slice::Iter<Word>,
) -> Result<Vec<Parameter>, ScriptError> {
    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {
        let word = expect_word("Unexpected end of script.", &mut iter)?;

        if first_param && word.kind == Some(Kind::ClosingParenthesis) {
            break;
        } else if word.kind == Some(Kind::Name) {
            first_param = false;

            parameters.push(Parameter::build_from_name(
                PositionnedString {
                    string: word.text,
                    position: word.position,
                },
                &mut iter,
            )?);

            let delimiter = expect_word("Unexpected end of script.", &mut iter)?;

            if delimiter.kind == Some(Kind::Comma) {
                continue;
            } else if delimiter.kind == Some(Kind::ClosingParenthesis) {
                break;
            } else {
                return Err(ScriptError::word(
                    "Comma or closing parenthesis expected.".to_string(),
                    delimiter.text,
                    delimiter.position,
                ));
            }
        } else {
            return Err(ScriptError::word(
                "Parameter declaration expected.".to_string(),
                word.text,
                word.position,
            ));
        }
    }

    Ok(parameters)
}

/// Build a parameter assignations list by parsing words.
///
/// * `iter`: Iterator over words list, next() being expected to be the the first parameter, _not_ parenthesis.
/// ```
/// # use melodium_lang::ScriptError;
/// # use melodium_lang::text::word::*;
/// # use melodium_lang::text::common::parse_parameters_assignations;
///
/// let text = r##"
/// (path = "my/path/to/something", sampleRate = 44100, frameSize = 4096, hopSize= 2048, windowingType="square")
/// "##;
///
/// let words = get_words(text).unwrap();
/// let mut iter = words.iter();
///
/// expect_word_kind(Kind::OpeningParenthesis, "Parameters declaration expected '('.", &mut iter)?;
/// let parameters = parse_parameters_assignations(&mut iter)?;
///
/// assert_eq!(parameters.len(), 5);
/// # Ok::<(), ScriptError>(())
/// ```
pub fn parse_parameters_assignations(
    mut iter: &mut std::slice::Iter<Word>,
) -> Result<Vec<Parameter>, ScriptError> {
    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {
        let word = expect_word("Unexpected end of script.", &mut iter)?;

        if first_param && word.kind == Some(Kind::ClosingParenthesis) {
            break;
        } else if word.kind == Some(Kind::Name) {
            first_param = false;

            expect_word_kind(Kind::Equal, "Parameter value expected.", &mut iter)?;
            parameters.push(Parameter::build_from_value(
                PositionnedString {
                    string: word.text,
                    position: word.position,
                },
                &mut iter,
            )?);

            let delimiter = expect_word("Unexpected end of script.", &mut iter)?;

            if delimiter.kind == Some(Kind::Comma) {
                continue;
            } else if delimiter.kind == Some(Kind::ClosingParenthesis) {
                break;
            } else {
                return Err(ScriptError::word(
                    "Comma or closing parenthesis expected.".to_string(),
                    delimiter.text,
                    delimiter.position,
                ));
            }
        } else {
            return Err(ScriptError::word(
                "Parameter declaration expected.".to_string(),
                word.text,
                word.position,
            ));
        }
    }

    Ok(parameters)
}

/// Build a configuration declaration list by parsing words.
///
/// * `iter`: Iterator over words list, next() being expected to be the first parameter, _not_ bracket.
/// ```
/// # use melodium_lang::error::ScriptError;
/// # use melodium_lang::text::word::*;
/// # use melodium_lang::text::common::parse_configuration_declarations;
///
/// let text = r##"
/// [Files: FileManager, Audio: AudioManager]
/// "##;
///
/// let words = get_words(text).unwrap();
/// let mut iter = words.iter();
///
/// expect_word_kind(Kind::OpeningBracket, "Models declaration expected '['.", &mut iter)?;
/// let config = parse_configuration_declarations(&mut iter)?;
///
/// assert_eq!(config.len(), 2);
/// # Ok::<(), ScriptError>(())
/// ```
pub fn parse_configuration_declarations(
    mut iter: &mut std::slice::Iter<Word>,
) -> Result<Vec<Parameter>, ScriptError> {
    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {
        let word = expect_word("Unexpected end of script.", &mut iter)?;

        if first_param && word.kind == Some(Kind::ClosingBracket) {
            break;
        } else if word.kind == Some(Kind::Name) {
            first_param = false;

            expect_word_kind(Kind::Colon, "Model type declaration expected.", &mut iter)?;
            parameters.push(Parameter::build_from_type(
                None,
                PositionnedString {
                    string: word.text,
                    position: word.position,
                },
                &mut iter,
            )?);

            let delimiter = expect_word("Unexpected end of script.", &mut iter)?;

            if delimiter.kind == Some(Kind::Comma) {
                continue;
            } else if delimiter.kind == Some(Kind::ClosingBracket) {
                break;
            } else {
                return Err(ScriptError::word(
                    "Comma or closing bracket expected.".to_string(),
                    delimiter.text,
                    delimiter.position,
                ));
            }
        } else {
            return Err(ScriptError::word(
                "Model declaration expected.".to_string(),
                word.text,
                word.position,
            ));
        }
    }

    Ok(parameters)
}

/// Build a configuration assignation list by parsing words.
///
/// * `iter`: Iterator over words list, next() being expected to be the first parameter, _not_ bracket.
/// ```
/// # use melodium_lang::ScriptError;
/// # use melodium_lang::text::word::*;
/// # use melodium_lang::text::common::parse_configuration_assignations;
///
/// let text = r##"
/// [Files=DataFiles, Audio=AudioConnection]
/// "##;
///
/// let words = get_words(text).unwrap();
/// let mut iter = words.iter();
///
/// expect_word_kind(Kind::OpeningBracket, "Models declaration expected '['.", &mut iter)?;
/// let config = parse_configuration_assignations(&mut iter)?;
///
/// assert_eq!(config.len(), 2);
/// # Ok::<(), ScriptError>(())
/// ```
pub fn parse_configuration_assignations(
    mut iter: &mut std::slice::Iter<Word>,
) -> Result<Vec<Parameter>, ScriptError> {
    let mut parameters = Vec::new();

    let mut first_param = true;
    loop {
        let word = expect_word("Unexpected end of script.", &mut iter)?;

        if first_param && word.kind == Some(Kind::ClosingBracket) {
            break;
        } else if word.kind == Some(Kind::Name) {
            first_param = false;

            expect_word_kind(Kind::Equal, "Assignation expected.", &mut iter)?;
            parameters.push(Parameter::build_from_value(
                PositionnedString {
                    string: word.text,
                    position: word.position,
                },
                &mut iter,
            )?);

            let delimiter = expect_word("Unexpected end of script.", &mut iter)?;

            if delimiter.kind == Some(Kind::Comma) {
                continue;
            } else if delimiter.kind == Some(Kind::ClosingBracket) {
                break;
            } else {
                return Err(ScriptError::word(
                    "Comma or closing bracket expected.".to_string(),
                    delimiter.text,
                    delimiter.position,
                ));
            }
        } else {
            return Err(ScriptError::word(
                "Configuration declaration expected.".to_string(),
                word.text,
                word.position,
            ));
        }
    }

    Ok(parameters)
}
