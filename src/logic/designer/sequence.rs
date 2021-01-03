
use std::rc::{Rc, Weak};
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

    model_instanciations: HashMap<String, Rc<RefCell<ModelInstanciation>>>,
    treatments: HashMap<String, Rc<RefCell<Treatment>>>,
    connections: HashSet<Rc<RefCell<Connection>>>,

    auto_reference: Weak<RefCell<Self>>,
}

impl Sequence {
    pub fn new(collections: &Rc<CollectionPool>, descriptor: &Rc<SequenceTreatmentDescriptor>) -> Rc<RefCell<Self>> {
        let sequence = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            collections: Rc::clone(collections),
            descriptor: Rc::clone(descriptor),
            model_instanciations: HashMap::new(),
            treatments: HashMap::new(),
            connections: HashSet::new(),
            auto_reference: Weak::new(),
        }));

        sequence.borrow_mut().auto_reference = Rc::downgrade(&sequence);

        sequence
    }

    pub fn collections(&self) -> &Rc<CollectionPool> {
        &self.collections
    }

    pub fn descriptor(&self) -> &Rc<SequenceTreatmentDescriptor> {
        &self.descriptor
    }

    pub fn add_model_intanciation(&mut self, model_identifier: &IdentifierDescriptor, name: &str) -> Result<(), LogicError> {
        todo!()
    }

    pub fn add_treatment(&mut self, identifier: &IdentifierDescriptor, name: &str) -> Result<Rc<RefCell<Treatment>>, LogicError> {
        
        if let Some(treatment_descriptor) = self.collections.treatments.get(identifier) {
            let treatment = Treatment::new(&self.auto_reference.upgrade().unwrap(), treatment_descriptor, name);
            let rc_treatment = Rc::new(RefCell::new(treatment));
            self.treatments.insert(name.to_string(), Rc::clone(&rc_treatment));
            Ok(rc_treatment)
        }
        else {
            Err(LogicError::unexisting_treatment())
        }

    }

    pub fn add_connection(&mut self, output_treatment: &str, output_name: &str, input_treatment: &str, intput_name: &str) -> Result<(), LogicError> {
        todo!()
    }

    pub fn add_input_connection(&mut self, self_input_name: &str, input_treatment: &str, input_name: &str) -> Result<(), LogicError> {
        todo!()
    }

    pub fn add_output_connection(&mut self, self_output_name: &str, output_treatment: &str, output_name: &str) -> Result<(), LogicError> {
        todo!()
    }

    pub fn model_instanciations(&self) -> &HashMap<String, Rc<RefCell<ModelInstanciation>>> {
        &self.model_instanciations
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

    pub fn register(&self) -> Result<(), LogicError> {

        self.validate()?;

        todo!()
    }
}

