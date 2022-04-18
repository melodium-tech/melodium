
use std::fmt::*;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
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
use super::super::builder::CoreTreatmentBuilder;
use crate::executive::world::{World, TrackId};
use crate::executive::treatment::Treatment as ExecutiveTreatment;

macro_rules! models {
    ($(($name:expr,$descriptor:expr)),*) => {
        vec![
            $(($name.to_string(),$descriptor),)*
        ]
    };
}
pub(crate) use models;

macro_rules! treatment_sources {
    () => {{
        std::collections::HashMap::new()
    }};
    ($(($descriptor:expr,$($source:expr),+)),*) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert(
            $descriptor,
            vec![
                $($source.to_string(),)+
            ]
        );)*
        map
    }};
}
pub(crate) use treatment_sources;

#[derive(Debug)]
pub struct CoreTreatment {
    identifier: Identifier,
    models: HashMap<String, Arc<CoreModel>>,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
    source_from: HashMap<Arc<CoreModel>, Vec<String>>,
    builder: Arc<Box<dyn Builder>>,
    auto_reference: Weak<Self>,
}

impl CoreTreatment {
    pub fn new(identifier: Identifier, models: Vec<(String, Arc<CoreModel>)>, source_from: HashMap<Arc<CoreModel>, Vec<String>>, parameters: Vec<Parameter>, inputs: Vec<Input>, outputs: Vec<Output>, new_treatment: fn(Arc<World>, TrackId) -> Arc<dyn ExecutiveTreatment>) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            identifier,
            models: HashMap::from_iter(models.iter().map(|m| (m.0.to_string(), Arc::clone(&m.1)))),
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
            inputs: HashMap::from_iter(inputs.iter().map(|i| (i.name().to_string(), i.clone()))),
            outputs: HashMap::from_iter(outputs.iter().map(|o| (o.name().to_string(), o.clone()))),
            source_from,
            builder: Arc::new(Box::new(CoreTreatmentBuilder::new(me.clone() as Weak<dyn Treatment>, new_treatment))),
            auto_reference: me.clone(),
        })
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

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.upgrade().unwrap()
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

    fn models(&self) -> &HashMap<String, Arc<CoreModel>> {
        &self.models
    }

    fn requirements(&self) -> &HashMap<String, Requirement> {
        
        lazy_static! {
            static ref EMPTY_REQUIREMENTS: HashMap<String, Requirement> = HashMap::default();
        }

        &EMPTY_REQUIREMENTS
    }

    fn source_from(&self) -> &HashMap<Arc<CoreModel>, Vec<String>> {
        &self.source_from
    }

    fn as_buildable(&self) -> Arc<dyn Buildable> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Display for CoreTreatment {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        writeln!(f, "Treatment `{}`", self.identifier.to_string())?;

        if !self.models.is_empty() {
            writeln!(f, "\nModels:")?;

            for model in &self.models {
                writeln!(f, "- _{}_: `{}`", model.0, model.1.identifier().to_string())?;
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

        Ok(())
        
    }
}

