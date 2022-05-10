
//! Module dedicated to [Function] parsing.

use crate::script::error::ScriptError;

use super::{PositionnedString, Position, Value};
use super::word::{expect_word, expect_word_kind, Kind, Word};

/// Structure describing a textual requirement.
/// 
/// It owns the requirement name.
#[derive(Clone, Debug)]
pub struct Function {
    pub name: PositionnedString,
    pub parameters: Vec<Value>,
}

impl Function {

    pub fn build_from_parameters(name: PositionnedString, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let mut parameters = Vec::new();

        let mut first_param = true;
        loop {
    
            // We _clone_ the iterator (in case next word is a value).
            let word = expect_word("Unexpected end of script.", &mut iter.clone())?;
    
            if first_param && word.kind == Some(Kind::ClosingParenthesis) {
                break;
            }
            else {
                first_param = false;

                iter.next();
    
                parameters.push(Value::build_from_first_item(&mut iter)?);
    
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
        }
    
        Ok(Self{
            name,
            parameters,
        })
    }
}
