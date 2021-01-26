
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use super::super::error::LogicError;
use super::super::collection_pool::CollectionPool;
use super::super::connections::Connections;
use super::super::IdentifierDescriptor;
use super::super::SequenceTreatmentDescriptor;
use super::super::TreatmentDescriptor;

use super::model_instanciation::ModelInstanciation;
use super::connection::Connection;
use super::treatment::Treatment;

pub struct Sequence {
    collections: Rc<CollectionPool>,
    descriptor: Rc<SequenceTreatmentDescriptor>,

    model_instanciations: HashMap<String, Rc<RefCell<ModelInstanciation>>>,
    treatments: HashMap<String, Rc<RefCell<Treatment>>>,
    connections: Vec<Rc<RefCell<Connection>>>,

    auto_reference: Weak<RefCell<Self>>,
}

impl Sequence {
    pub fn new(collections: &Rc<CollectionPool>, descriptor: &Rc<SequenceTreatmentDescriptor>) -> Rc<RefCell<Self>> {
        let sequence = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            collections: Rc::clone(collections),
            descriptor: Rc::clone(descriptor),
            model_instanciations: HashMap::new(),
            treatments: HashMap::new(),
            connections: Vec::new(),
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

    pub fn add_model_intanciation(&mut self, model_identifier: &IdentifierDescriptor, name: &str) -> Result<Rc<RefCell<ModelInstanciation>>, LogicError> {
        
        if let Some(model_descriptor) = self.collections.models.get(model_identifier) {
            let model = ModelInstanciation::new(&self.auto_reference.upgrade().unwrap(), model_descriptor, name);
            let rc_model = Rc::new(RefCell::new(model));
            self.model_instanciations.insert(name.to_string(), Rc::clone(&rc_model));
            Ok(rc_model)
        }
        else {
            Err(LogicError::unexisting_model())
        }
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

    pub fn add_connection(&mut self, output_treatment: &str, output_name: &str, input_treatment: &str, input_name: &str) -> Result<(), LogicError> {
        
        let rc_output_treatment;
        let datatype_output;
        if let Some(pos_rc_output_treatment) = self.treatments.get(output_treatment) {
            rc_output_treatment = pos_rc_output_treatment;

            if let Some(pos_output) = rc_output_treatment.borrow().descriptor().outputs().get(output_name) {
                datatype_output = pos_output.datatype().clone();
            }
            else {
                return Err(LogicError::connection_output_not_found())
            }
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        let rc_input_treatment;
        let datatype_input;
        if let Some(pos_rc_input_treatment) = self.treatments.get(input_treatment) {
            rc_input_treatment = pos_rc_input_treatment;

            if let Some(pos_input) = rc_input_treatment.borrow().descriptor().inputs().get(input_name) {
                datatype_input = pos_input.datatype().clone();
            }
            else {
                return Err(LogicError::connection_input_not_found())
            }
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        if let Some(arc_connection_descriptor) = Connections::get(Some(datatype_output), Some(datatype_input)) {

            let mut connection = Connection::new(&self.auto_reference.upgrade().unwrap(), arc_connection_descriptor);

            connection.set_output(rc_output_treatment, Some(output_name))?;

            connection.set_input(rc_input_treatment, Some(input_name))?;

            connection.validate()?;

            let rc_connection = Rc::new(RefCell::new(connection));
            self.connections.push(Rc::clone(&rc_connection));

            Ok(())
        }
        else {
            return Err(LogicError::unexisting_connexion_type())
        }
    }

    pub fn add_input_connection(&mut self, self_input_name: &str, input_treatment: &str, input_name: &str) -> Result<(), LogicError> {
        
        let datatype_input_self;
        if let Some(pos_input) = self.descriptor.inputs().get(self_input_name) {
            datatype_input_self = pos_input.datatype().clone();
        }
        else {
            return Err(LogicError::connection_self_input_not_found())
        }

        let rc_input_treatment;
        let datatype_input_treatment;
        if let Some(pos_rc_input_treatment) = self.treatments.get(input_treatment) {
            rc_input_treatment = pos_rc_input_treatment;

            if let Some(pos_input) = rc_input_treatment.borrow().descriptor().inputs().get(input_name) {
                datatype_input_treatment = pos_input.datatype().clone();
            }
            else {
                return Err(LogicError::connection_input_not_found())
            }
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        if let Some(arc_connection_descriptor) = Connections::get(Some(datatype_input_self), Some(datatype_input_treatment)) {

            let mut connection = Connection::new(&self.auto_reference.upgrade().unwrap(), arc_connection_descriptor);

            connection.set_self_input(Some(self_input_name))?;

            connection.set_input(rc_input_treatment, Some(input_name))?;

            connection.validate()?;

            let rc_connection = Rc::new(RefCell::new(connection));
            self.connections.push(Rc::clone(&rc_connection));

            Ok(())
        }
        else {
            return Err(LogicError::unexisting_connexion_type())
        }
    }

    pub fn add_output_connection(&mut self, self_output_name: &str, output_treatment: &str, output_name: &str) -> Result<(), LogicError> {
        
        let datatype_output_self;
        if let Some(pos_output) = self.descriptor.outputs().get(self_output_name) {
            datatype_output_self = pos_output.datatype().clone();
        }
        else {
            return Err(LogicError::connection_self_output_not_found())
        }

        let rc_output_treatment;
        let datatype_output_treatment;
        if let Some(pos_rc_output_treatment) = self.treatments.get(output_treatment) {
            rc_output_treatment = pos_rc_output_treatment;

            if let Some(pos_output) = rc_output_treatment.borrow().descriptor().outputs().get(output_name) {
                datatype_output_treatment = pos_output.datatype().clone();
            }
            else {
                return Err(LogicError::connection_output_not_found())
            }
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        if let Some(arc_connection_descriptor) = Connections::get(Some(datatype_output_treatment), Some(datatype_output_self)) {

            let mut connection = Connection::new(&self.auto_reference.upgrade().unwrap(), arc_connection_descriptor);

            connection.set_self_input(Some(self_output_name))?;

            connection.set_input(rc_output_treatment, Some(output_name))?;

            connection.validate()?;

            let rc_connection = Rc::new(RefCell::new(connection));
            self.connections.push(Rc::clone(&rc_connection));

            Ok(())
        }
        else {
            return Err(LogicError::unexisting_connexion_type())
        }
    }

    pub fn model_instanciations(&self) -> &HashMap<String, Rc<RefCell<ModelInstanciation>>> {
        &self.model_instanciations
    }

    pub fn treatments(&self) -> &HashMap<String, Rc<RefCell<Treatment>>> {
        &self.treatments
    }

    pub fn connections(&self) -> &Vec<Rc<RefCell<Connection>>> {
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

