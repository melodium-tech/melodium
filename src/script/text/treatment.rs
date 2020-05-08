
use crate::script::error::ScriptError;

use super::word::{expect_word, Kind, Word};
use super::parameter::Parameter;

pub struct Treatment {
    pub name: String,
    pub r#type: String,
    pub parameters: Vec<Parameter>,
}

impl Treatment {
    pub fn build(name: String, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let mut r#type: Option<String> = None;
        let mut parameters = Vec::new();

        let mut first_param = true;

        loop {

            let word = expect_word("Unexpected end of script.", &mut iter)?;

            if first_param {
                if word.kind == Some(Kind::ClosingParenthesis) {
                    break;
                }
            }

            if word.kind == Some(Kind::Name) {
                let determinant = expect_word("Unexpected end of script.", &mut iter)?;
                let delimiter;

                if determinant.kind == Some(Kind::Equal) {
                    parameters.push(Parameter::build_from_value(word.text, &mut iter)?);
                    delimiter = expect_word("Unexpected end of script.", &mut iter)?;
                }
                else if first_param {
                    r#type = Some(word.text);
                    delimiter = determinant;
                }
                else {
                    return Err(ScriptError::new("Parameter value assignation expected.".to_string(), determinant.text, determinant.line, determinant.line_position, determinant.absolute_position));
                }

                first_param = false;
                
                if delimiter.kind == Some(Kind::Comma) {
                    continue;
                }
                else if delimiter.kind == Some(Kind::ClosingParenthesis) {
                    break;
                }
                else {
                    return Err(ScriptError::new("Comma or closing parenthesis expected.".to_string(), delimiter.text, delimiter.line, delimiter.line_position, delimiter.absolute_position));
                }
            }
            else {
                return Err(ScriptError::new("Parameter or closing parenthesis expected.".to_string(), word.text, word.line, word.line_position, word.absolute_position));
            }
        }

        if r#type.is_none() {
            r#type = Some(name.to_string());
        }

        Ok(Self {
            name: name,
            r#type: r#type.unwrap(),
            parameters: parameters,
        })
    }
}
