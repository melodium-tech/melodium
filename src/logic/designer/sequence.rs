
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use super::super::error::LogicError;
use super::super::collection_pool::CollectionPool;
use super::super::IdentifierDescriptor;
use super::super::SequenceTreatmentDescriptor;

use super::model_instanciation::ModelInstanciation;
use super::connection::Connection;
use super::treatment::Treatment;

pub struct Sequence {
    collections: Rc<CollectionPool>,
    descriptor: Rc<SequenceTreatmentDescriptor>,

    model_instanciations: HashMap<String, ModelInstanciation>,
    treatments: HashMap<String, Rc<RefCell<Treatment>>>,
    connections: HashSet<Rc<RefCell<Connection>>>,
}

impl Sequence {
    pub fn new(collections: &Rc<CollectionPool>, descriptor: &Rc<SequenceTreatmentDescriptor>) -> Self {
        Self {
            collections: Rc::clone(collections),
            descriptor: Rc::clone(descriptor),
            model_instanciations: HashMap::new(),
            treatments: HashMap::new(),
            connections: HashSet::new(),
        }
    }

    pub fn collections(&self) -> &Rc<CollectionPool> {
        &self.collections
    }

    pub fn descriptor(&self) -> &Rc<SequenceTreatmentDescriptor> {
        &self.descriptor
    }

    pub fn add_treatment(&mut self, identifier: &IdentifierDescriptor, name: &str) -> Result<(), LogicError> {
        
        if let Some(treatment_descriptor) = self.collections.treatments.get(identifier) {
            let treatment = Treatment::new(treatment_descriptor, name);
            self.treatments.insert(name.to_string(), Rc::new(RefCell::new(treatment)));
            Ok(())
        }
        else {
            // TODO
            Err(LogicError{})
        }

    }

    pub fn add_model_intanciation(&mut self, model_identifier: &IdentifierDescriptor, name: &str) -> Result<(), LogicError> {
        // TODO
        Err(LogicError{})
    }

    pub fn add_connection(&mut self, output_treatment: &str, output_name: &str, input_treatment: &str, intput_name: &str) -> Result<(), LogicError> {
        // TODO
        Err(LogicError{})
    }

    pub fn add_input_connection(&mut self, self_input_name: &str, input_treatment: &str, input_name: &str) -> Result<(), LogicError> {
        // TODO
        Err(LogicError{})
    }

    pub fn add_out_connection(&mut self, self_output_name: &str, output_treatment: &str, output_name: &str) -> Result<(), LogicError> {
        // TODO
        Err(LogicError{})
    }

    pub fn treatments(&self) -> &HashMap<String, Rc<RefCell<Treatment>>> {
        &self.treatments
    }

    pub fn connections(&self) -> &HashSet<Rc<RefCell<Connection>>> {
        &self.connections
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        Ok(())
    }
}

