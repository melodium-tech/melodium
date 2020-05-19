
use crate::script::error::ScriptError;

use super::word::*;
use super::r#type::Type;
use super::value::Value;

pub struct Parameter {
    pub name: String,
    pub r#type: Option<Type>,
    pub value: Option<Value>,
}

impl Parameter {
    pub fn build_from_type(name: String, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let r#type = Type::build(&mut iter)?;

        // We _clone_ the iterator (in case next word doesn't rely on Parameter) and doesn't make our expectation to fail if not satisfied.
        let possible_equal = expect_word_kind(Kind::Equal, "", &mut iter.clone());
        if possible_equal.is_ok() {
            // We discard the equal sign.
            iter.next();

            let value = Value::build_from_first_item(&mut iter)?;

            Ok(Self {
                name,
                r#type: Some(r#type),
                value: Some(value),
            })
        }
        else {
            Ok(Self {
                name,
                r#type: Some(r#type),
                value: None,
            })
        }
    }

    pub fn build_from_value(name: String, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let value = Value::build_from_first_item(&mut iter)?;

        Ok(Self {
            name,
            r#type: None,
            value: Some(value),
        })
    }
}
