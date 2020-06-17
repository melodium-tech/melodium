
use crate::script::error::ScriptError;
use crate::script::text::Sequence as TextSequence;

use super::script::Script;
use super::declared_parameter::DeclaredParameter;
use super::requirement::Requirement;
use super::input::Input;
use super::output::Output;
use super::treatment::Treatment;
use super::connection::Connection;

pub struct Sequence<'a> {
    pub text: TextSequence,

    pub script: &'a Script<'a>,

    pub name: String,

    pub parameters: Vec<DeclaredParameter<'a>>,
    pub requirements: Vec<Requirement<'a>>,
    pub origin: Option<Treatment<'a>>,
    pub inputs: Vec<Input<'a>>,
    pub outputs: Vec<Output<'a>>,
    pub treatments: Vec<Treatment<'a>>,
    pub connections: Vec<Connection<'a>>
}

impl<'a> Sequence<'a> {
    pub fn new(script: &'a Script, text: TextSequence) -> Result<Self, ScriptError> {

        let mut sequence = Self {
            text,
            script,
            name: text.name,
            parameters: Vec::new(),
            requirements: Vec::new(),
            origin: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            treatments: Vec::new(),
            connections: Vec::new(),
        };

        {
            let sequence = script.find_sequence(&text.name);
            if sequence.is_some() {
                return Err(ScriptError::semantic("Sequence '".to_string() + &text.name + "' is already declared."))
            }

            let r#use = script.find_use(&text.name);
            if r#use.is_some() {
                return Err(ScriptError::semantic("Element '".to_string() + &text.name + "' is already declared as used."))
            }
        }

        for p in text.parameters {
            sequence.parameters.push(DeclaredParameter::new(&sequence, p)?);
        }

        for r in text.requirements {
            sequence.requirements.push(Requirement::new(&sequence, r)?);
        }

        if text.origin.is_some() {
            sequence.origin = Some(Treatment::new(&sequence, text.origin.unwrap())?);
        }

        for i in text.inputs {
            sequence.inputs.push(Input::new(&sequence, i)?);
        }

        for o in text.outputs {
            sequence.outputs.push(Output::new(&sequence, o)?);
        }

        for t in text.treatments {
            sequence.treatments.push(Treatment::new(&sequence, t)?);
        }

        for c in text.connections {
            sequence.connections.push(Connection::new(&sequence, c)?);
        }

        Ok(sequence)
    }

    pub fn find_parameter(&self, name: & str) -> Option<&DeclaredParameter> {
        self.parameters.iter().find(|&p| p.name == name)
    }

    pub fn find_requirement(&self, name: & str) -> Option<&Requirement> {
        self.requirements.iter().find(|&r| r.name == name) 
    }

    pub fn find_input(&self, name: & str) -> Option<&Input> {
        self.inputs.iter().find(|&i| i.name == name) 
    }

    pub fn find_output(&self, name: & str) -> Option<&Output> {
        self.outputs.iter().find(|&o| o.name == name) 
    }

    pub fn find_treatment(&self, name: & str) -> Option<&Treatment> {
        self.treatments.iter().find(|&t| t.name == name) 
    }
}