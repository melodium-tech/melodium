
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
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
use crate::executive::world::World;
use crate::executive::treatment::Treatment as ExecutiveTreatment;

#[derive(Debug)]
pub struct CoreTreatment {
    identifier: Identifier,
    models: HashMap<String, Arc<CoreModel>>,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
    source_from: HashMap<Arc<CoreModel>, Vec<String>>,
    builder: RwLock<Option<Arc<Box<dyn Builder>>>>,
    auto_reference: RwLock<Weak<Self>>,
}

//, builder: Box<dyn Builder>

impl CoreTreatment {
    pub fn new(identifier: Identifier, models: Vec<(String, Arc<CoreModel>)>, source_from: HashMap<Arc<CoreModel>, Vec<String>>, parameters: Vec<Parameter>, inputs: Vec<Input>, outputs: Vec<Output>, new_treatment: fn(Arc<World>) -> Arc<dyn ExecutiveTreatment>) -> Arc<Self> {
        let mut descriptor = Arc::new(Self{
            identifier,
            models: HashMap::from_iter(models.iter().map(|m| (m.0.to_string(), Arc::clone(&m.1)))),
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
            inputs: HashMap::from_iter(inputs.iter().map(|i| (i.name().to_string(), i.clone()))),
            outputs: HashMap::from_iter(outputs.iter().map(|o| (o.name().to_string(), o.clone()))),
            source_from,
            builder: RwLock::new(None),
            auto_reference: RwLock::new(Weak::new()),
        });

        let rc_descriptor = Arc::clone(&descriptor);
        let rc_descriptor_treatment: Arc<dyn Treatment> = Arc::clone(&(descriptor as Arc<dyn Treatment>));
        *rc_descriptor.builder.write().unwrap() = Some(Arc::new(Box::new(CoreTreatmentBuilder::new(&rc_descriptor_treatment, new_treatment))));

        *rc_descriptor.auto_reference.write().unwrap() = Arc::downgrade(&rc_descriptor);

        rc_descriptor
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
        self.auto_reference.read().unwrap().upgrade().unwrap()
    }
}

impl Buildable for CoreTreatment {

    fn builder(&self) -> Arc<Box<dyn Builder>> {
        Arc::clone(&self.builder.read().unwrap().as_ref().unwrap())
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
        self.auto_reference.read().unwrap().upgrade().unwrap()
    }
}
