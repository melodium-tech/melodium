
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::sequence::Sequence;
use super::r#type::Type;
use super::value::Value;

pub struct DeclaredParameter<'a> {
    pub text: TextParameter,

    pub sequence: &'a Sequence<'a>,

    pub name: String,
    pub r#type: Type,
    pub value: Option<Value>,
}

impl<'a> DeclaredParameter<'a> {
    pub fn new(sequence: &'a Sequence, text: TextParameter) -> Result<Self, ScriptError> {

        let parameter = sequence.find_parameter(&text.name);
        if parameter.is_some() {
            return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' is already declared."))
        }

        if text.r#type.is_none() {
            return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' do not have type."))
        }
        let r#type = Type::new(text.r#type.unwrap())?;

        let mut value = None;
        if text.value.is_some() {
            value = Some(Value::new(text.value.unwrap())?);
        }

        Ok(Self {
            text,
            sequence,
            name: text.name,
            r#type,
            value,
        })
    }
}

