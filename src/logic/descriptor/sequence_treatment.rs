
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::designable::Designable;
use super::buildable::Buildable;
use super::super::builder::Builder;
use super::input::Input;
use super::output::Output;
use super::core_model::CoreModel;
use super::parameter::Parameter;
use super::requirement::Requirement;
use super::treatment::Treatment;

#[derive(Debug)]
pub struct SequenceTreatment {
    identifier: Identifier,
    models: HashMap<String, Arc<CoreModel>>,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
    requirements: HashMap<String, Requirement>,
    source_from: HashMap<Arc<CoreModel>, Vec<String>>,
    builder: RwLock<Option<Arc<Box<dyn Builder>>>>,
    auto_reference: RwLock<Weak<Self>>,
}

impl SequenceTreatment {
    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            models: HashMap::new(),
            parameters: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            requirements: HashMap::new(),
            source_from: HashMap::new(),
            builder: RwLock::new(None),
            auto_reference: RwLock::new(Weak::new()),
        }
    }

    pub fn set_autoref(&self, reference: &Arc<Self>) {
        *self.auto_reference.write().unwrap() = Arc::downgrade(reference);
    }

    pub fn add_model(&mut self, name: &str, model: &Arc<CoreModel>) {
        self.models.insert(name.to_string(), Arc::clone(model));
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

impl Parameterized for SequenceTreatment {

    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.read().unwrap().upgrade().unwrap()
    }
}

impl Treatment for SequenceTreatment {

    fn inputs(&self) -> &HashMap<String, Input> {
        &self.inputs
    }

    fn outputs(&self) -> &HashMap<String, Output> {
        &self.outputs
    }

    fn models(&self) -> &HashMap<String, Arc<CoreModel>> {
        &self.models
    }

    fn requirements(&self) -> &HashMap<String, Requirement> {
        &self.requirements
    }

    fn source_from(&self) -> &HashMap<Arc<CoreModel>, Vec<String>> {
        // Always empty
        &self.source_from
    }

    fn as_buildable(&self) -> Arc<dyn Buildable> {
        self.auto_reference.read().unwrap().upgrade().unwrap()
    }
}

impl Designable for SequenceTreatment {
    
    fn register_builder(&self, builder: Box<dyn Builder>) {
        *(self.builder.write().unwrap()) = Some(Arc::new(builder))
    }
}

impl Buildable for SequenceTreatment {
    
    fn builder(&self) -> Arc<Box<dyn Builder>> {
        Arc::clone(self.builder.read().unwrap().as_ref().unwrap())
    }
}
