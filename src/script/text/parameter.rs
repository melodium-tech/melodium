
use crate::script::error::ScriptError;

use super::word::*;
use super::value::Value;

pub struct Parameter {
    pub name: String,
    pub value: Value,
}

impl Parameter {
    pub fn build_from_value(name: String, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let value = Value::build_from_first_item(&mut iter)?;

        Ok(Self {
            name,
            value,
        })
    }
}
