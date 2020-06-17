
use crate::script::error::ScriptError;
use crate::script::text::Requirement as TextRequirement;

use super::sequence::Sequence;

pub struct Requirement<'a> {
    pub text: TextRequirement,

    pub sequence: &'a Sequence<'a>,

    pub name: String,
}

impl<'a> Requirement<'a> {
    pub fn new(sequence: &'a Sequence, text: TextRequirement) -> Result<Self, ScriptError> {

        let requirement = sequence.find_requirement(&text.name);
        if requirement.is_some() {
            return Err(ScriptError::semantic("'".to_string() + &text.name + "' is already required."))
        }

        Ok(Self{
            text,
            sequence,
            name: text.name,
        })
    }
}
