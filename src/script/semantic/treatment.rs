
use crate::script::error::ScriptError;
use crate::script::text::Treatment as TextTreatment;

use super::sequence::Sequence;
use super::assigned_parameter::AssignedParameter;

pub struct Treatment<'a> {
    pub text: TextTreatment,

    pub sequence: &'a Sequence<'a>,

    pub name: String,
    pub r#type: String,
    pub parameters: Vec<AssignedParameter<'a>>,
}

impl<'a> Treatment<'a> {
    pub fn new(sequence: &'a Sequence, text: TextTreatment) -> Result<Self, ScriptError> {

        let mut treatment = Self {
            text,
            sequence,
            name: text.name,
            r#type: text.r#type,
            parameters: Vec::new(),
        };

        // Maybe not good to check that.
        let r#use = sequence.script.find_use(&text.r#type);
        if r#use.is_none() {
            return Err(ScriptError::semantic("'".to_string() + &text.r#type + "' is unkown."))
        }

        {
            let treatment = sequence.find_treatment(&text.name);
            if treatment.is_some() {
                return Err(ScriptError::semantic("Treatment '".to_string() + &text.name + "' is already declared."))
            }
        }

        for p in text.parameters {
            treatment.parameters.push(AssignedParameter::new(&treatment, p)?);
        }

        Ok(treatment)

    }

    pub fn find_parameter(&self, name: & str) -> Option<&AssignedParameter> {
        self.parameters.iter().find(|&p| p.name == name) 
    }
}