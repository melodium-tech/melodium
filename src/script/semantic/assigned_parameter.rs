
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::treatment::Treatment;
use super::value::Value;

pub struct AssignedParameter<'a> {
    pub text: TextParameter,

    pub treatment: &'a Treatment<'a>,

    pub name: String,
    pub value: Value,
}

impl<'a> AssignedParameter<'a> {
    pub fn new(treatment: &'a Treatment, text: TextParameter) -> Result<Self, ScriptError> {

        let parameter = treatment.find_parameter(&text.name);
        if parameter.is_some() {
            return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' is already assigned."))
        }

        let value;
        if text.value.is_some() {
            value = Value::new(text.value.unwrap())?;
        }
        else {
            return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' is missing value."))
        }

        Ok(Self {
            text,
            treatment,
            name: text.name,
            value,
        })
    }

}