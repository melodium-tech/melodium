
use std::collections::HashMap;
use std::rc::Rc;
use std::iter::FromIterator;
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::input::Input;
use super::output::Output;
use super::core_model::CoreModel;
use super::parameter::Parameter;
use super::requirement::Requirement;
use super::treatment::Treatment;

#[derive(Debug)]
pub struct CoreTreatment {
    identifier: Identifier,
    models: HashMap<String, Rc<CoreModel>>,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
}

impl CoreTreatment {
    pub fn new(identifier: Identifier, models: Vec<(String, Rc<CoreModel>)>, parameters: Vec<Parameter>, inputs: Vec<Input>, outputs: Vec<Output>) -> Self {
        Self {
            identifier,
            models: HashMap::from_iter(models.iter().map(|m| (m.0.to_string(), Rc::clone(&m.1)))),
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
            inputs: HashMap::from_iter(inputs.iter().map(|i| (i.name().to_string(), i.clone()))),
            outputs: HashMap::from_iter(outputs.iter().map(|o| (o.name().to_string(), o.clone()))),
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
