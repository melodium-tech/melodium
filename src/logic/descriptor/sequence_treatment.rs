
use std::fmt::*;
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::designable::Designable;
use super::buildable::Buildable;
use super::super::designer::SequenceDesigner;
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
    designer: RwLock<Option<Arc<RwLock<SequenceDesigner>>>>,
    builder: RwLock<Option<Arc<Box<dyn Builder>>>>,
    auto_reference: Weak<Self>,
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
            designer: RwLock::new(None),
            builder: RwLock::new(None),
            auto_reference: Weak::default(),
        }
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

    pub fn commit(self) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            identifier: self.identifier,
            models: self.models,
            parameters: self.parameters,
            inputs: self.inputs,
            outputs: self.outputs,
            requirements: self.requirements,
            source_from: self.source_from,
            designer: self.designer,
            builder: self.builder,
            auto_reference: me.clone(),
        })
    }

    pub fn set_designer(&self, designer: Arc<RwLock<SequenceDesigner>>) {

        *self.designer.write().unwrap() = Some(designer);
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
        self.auto_reference.upgrade().unwrap()
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

    fn designer(&self) -> Option<Arc<RwLock<SequenceDesigner>>> {
        Some(Arc::clone(self.designer.read().unwrap().as_ref().unwrap()))
    }

    fn as_buildable(&self) -> Arc<dyn Buildable> {
        self.auto_reference.upgrade().unwrap()
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

impl Display for SequenceTreatment {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        writeln!(f, "Sequence `{}`", self.identifier.to_string())?;

        if !self.models.is_empty() {
            writeln!(f, "\nModels:")?;

            for model in &self.models {
                writeln!(f, "- {}: `{}`", model.0, model.1.identifier().to_string())?;
            }
        }

        if !self.parameters.is_empty() {
            writeln!(f, "\nParameters:")?;

            for parameter in &self.parameters {
                writeln!(f, "- {}", parameter.1)?;
            }
        }

        if !self.inputs.is_empty() {
            writeln!(f, "\nInputs:")?;

            for input in &self.inputs {
                writeln!(f, "- {}", input.1)?;
            }
        }

        if !self.outputs.is_empty() {
            writeln!(f, "\nOutputs:")?;

            for output in &self.outputs {
                writeln!(f, "- {}", output.1)?;
            }
        }

        if !self.requirements.is_empty() {
            writeln!(f, "\nRequire:")?;

            for require in &self.requirements {
                writeln!(f, "- {}", require.1.name())?;
            }
        }

        Ok(())
        
    }
}

