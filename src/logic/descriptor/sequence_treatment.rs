
use std::collections::HashMap;
use super::identified::Identified;
use super::identifier::Identifier;
use super::input::Input;
use super::output::Output;
use super::parameter::Parameter;
use super::requirement::Requirement;
use super::treatment::Treatment;

pub struct SequenceTreatment {
    identifier: Identifier,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
    requirements: HashMap<String, Requirement>,
}

impl SequenceTreatment {
    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            parameters: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            requirements: HashMap::new()
        }
    }

    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.insert(parameter.name().to_string(), parameter);
    }

    pub fn add_input(&mut self, input: Input) {
        self.inputs.insert(input.name().to_string(), input);
    }

    pub fn add_output(&mut self, output: Output) {
        self.outputs.insert(output.name().to_string(), output);
    }

    pub fn add_requirement(&mut self, requirement: Requirement) {
        self.requirements.insert(requirement.name().to_string(), requirement);
    }
}

impl Identified for SequenceTreatment {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Treatment for SequenceTreatment {

    fn inputs(&self) -> &HashMap<String, Input> {
        &self.inputs
    }

    fn outputs(&self) -> &HashMap<String, Output> {
        &self.outputs
    }

    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn requirements(&self) -> &HashMap<String, Requirement> {
        &self.requirements
    }
}
