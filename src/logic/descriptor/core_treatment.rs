
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::iter::FromIterator;
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::buildable::Buildable;
use super::input::Input;
use super::output::Output;
use super::core_model::CoreModel;
use super::parameter::Parameter;
use super::requirement::Requirement;
use super::treatment::Treatment;
use super::super::builder::Builder;

#[derive(Debug)]
pub struct CoreTreatment {
    identifier: Identifier,
    models: HashMap<String, Rc<CoreModel>>,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
    builder: Arc<Box<dyn Builder>>,
}

impl CoreTreatment {
    pub fn new(identifier: Identifier, models: Vec<(String, Rc<CoreModel>)>, parameters: Vec<Parameter>, inputs: Vec<Input>, outputs: Vec<Output>, builder: Box<dyn Builder>) -> Self {
        Self {
            identifier,
            models: HashMap::from_iter(models.iter().map(|m| (m.0.to_string(), Rc::clone(&m.1)))),
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
            inputs: HashMap::from_iter(inputs.iter().map(|i| (i.name().to_string(), i.clone()))),
            outputs: HashMap::from_iter(outputs.iter().map(|o| (o.name().to_string(), o.clone()))),
            builder: Arc::new(builder),
        }
    }

}

impl Identified for CoreTreatment {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Parameterized for CoreTreatment {

    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }
}

impl Buildable for CoreTreatment {

    fn builder(&self) -> Arc<Box<dyn Builder>> {
        Arc::clone(&self.builder)
    }
}

impl Treatment for CoreTreatment {

    fn inputs(&self) -> &HashMap<String, Input> {
        &self.inputs
    }

    fn outputs(&self) -> &HashMap<String, Output> {
        &self.outputs
    }

    fn models(&self) -> &HashMap<String, Rc<CoreModel>> {
        &self.models
    }

    fn requirements(&self) -> &HashMap<String, Requirement> {
        
        lazy_static! {
            static ref EMPTY_REQUIREMENTS: HashMap<String, Requirement> = HashMap::default();
        }

        &EMPTY_REQUIREMENTS
    }
}
