
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::sequence::Sequence;
use super::r#type::Type;

pub struct Output<'a> {
    pub text: TextParameter,

    pub sequence: &'a Sequence<'a>,

    pub name: String,
    pub r#type: Type,
}

impl<'a> Output<'a> {
    pub fn new(sequence: &'a Sequence, text: TextParameter) -> Result<Self, ScriptError> {

        let input = sequence.find_output(&text.name);
        if input.is_some() {
            return Err(ScriptError::semantic("Output '".to_string() + &text.name + "' is already declared."))
        }

        if text.r#type.is_none() {
            return Err(ScriptError::semantic("Output '".to_string() + &text.name + "' do not have type."))
        }
        let r#type = Type::new(text.r#type.unwrap())?;

        if text.value.is_some() {
            return Err(ScriptError::semantic("Output '".to_string() + &text.name + "' cannot have default value."))
        }

        Ok(Self{
            text,
            sequence,
            name: text.name,
            r#type,
        })
    }
}

