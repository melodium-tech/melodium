
use std::sync::{Arc, Weak, RwLock};
use std::collections::HashMap;
use super::super::error::LogicError;
use super::super::collection_pool::CollectionPool;
use super::super::connections::Connections;
use super::super::descriptor::IdentifierDescriptor;
use super::super::descriptor::SequenceTreatmentDescriptor;
use super::super::descriptor::DesignableDescriptor;
use super::super::descriptor::TreatmentDescriptor;

use super::model_instanciation::ModelInstanciation;
use super::connection::{Connection, IO};
use super::treatment::Treatment;

use super::super::builder::sequence_builder::SequenceBuilder;

#[derive(Debug)]
pub struct Sequence {
    collections: Arc<CollectionPool>,
    descriptor: Arc<SequenceTreatmentDescriptor>,

    model_instanciations: HashMap<String, Arc<RwLock<ModelInstanciation>>>,
    treatments: HashMap<String, Arc<RwLock<Treatment>>>,
    connections: Vec<Arc<RwLock<Connection>>>,

    auto_reference: Weak<RwLock<Self>>,
}

impl Sequence {
    pub fn new(collections: &Arc<CollectionPool>, descriptor: &Arc<SequenceTreatmentDescriptor>) -> Arc<RwLock<Self>> {
        let sequence = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            collections: Arc::clone(collections),
            descriptor: Arc::clone(descriptor),
            model_instanciations: HashMap::new(),
            treatments: HashMap::new(),
            connections: Vec::new(),
            auto_reference: Weak::new(),
        }));

        sequence.write().unwrap().auto_reference = Arc::downgrade(&sequence);

        sequence
    }

    pub fn collections(&self) -> &Arc<CollectionPool> {
        &self.collections
    }

    pub fn descriptor(&self) -> &Arc<SequenceTreatmentDescriptor> {
        &self.descriptor
    }

    pub fn add_model_instanciation(&mut self, model_identifier: &IdentifierDescriptor, name: &str) -> Result<Arc<RwLock<ModelInstanciation>>, LogicError> {
        
        if let Some(model_descriptor) = self.collections.models.get(model_identifier) {
            let model = ModelInstanciation::new(&self.auto_reference.upgrade().unwrap(), model_descriptor, name);
            let rc_model = Arc::new(RwLock::new(model));
            self.model_instanciations.insert(name.to_string(), Arc::clone(&rc_model));
            Ok(rc_model)
        }
        else {
            Err(LogicError::unexisting_model())
        }
    }

    pub fn add_treatment(&mut self, identifier: &IdentifierDescriptor, name: &str) -> Result<Arc<RwLock<Treatment>>, LogicError> {
        
        if let Some(treatment_descriptor) = self.collections.treatments.get(identifier) {
            let rc_treatment = Treatment::new(&self.auto_reference.upgrade().unwrap(), treatment_descriptor, name);
            self.treatments.insert(name.to_string(), Arc::clone(&rc_treatment));
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

            if let Some(pos_output) = rc_output_treatment.read().unwrap().descriptor().outputs().get(output_name) {
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

            if let Some(pos_input) = rc_input_treatment.read().unwrap().descriptor().inputs().get(input_name) {
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

            let rc_connection = Arc::new(RwLock::new(connection));
            self.connections.push(Arc::clone(&rc_connection));

            Ok(())
        }
        else {
            return Err(LogicError::unexisting_connexion_type())
        }
    }

    pub fn add_void_connection(&mut self, output_treatment: &str, input_treatment: &str) -> Result<(), LogicError> {

        let rc_output_treatment;
        if let Some(pos_rc_output_treatment) = self.treatments.get(output_treatment) {
            rc_output_treatment = pos_rc_output_treatment;
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        let rc_input_treatment;
        if let Some(pos_rc_input_treatment) = self.treatments.get(input_treatment) {
            rc_input_treatment = pos_rc_input_treatment;
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        if let Some(arc_connection_descriptor) = Connections::get(None, None) {

            let mut connection = Connection::new(&self.auto_reference.upgrade().unwrap(), arc_connection_descriptor);

            connection.set_output(rc_output_treatment, None)?;

            connection.set_input(rc_input_treatment, None)?;

            connection.validate()?;

            let rc_connection = Arc::new(RwLock::new(connection));
            self.connections.push(Arc::clone(&rc_connection));

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

            if let Some(pos_input) = rc_input_treatment.read().unwrap().descriptor().inputs().get(input_name) {
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

            let rc_connection = Arc::new(RwLock::new(connection));
            self.connections.push(Arc::clone(&rc_connection));

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

            if let Some(pos_output) = rc_output_treatment.read().unwrap().descriptor().outputs().get(output_name) {
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

            let rc_connection = Arc::new(RwLock::new(connection));
            self.connections.push(Arc::clone(&rc_connection));

            Ok(())
        }
        else {
            return Err(LogicError::unexisting_connexion_type())
        }
    }

    pub fn model_instanciations(&self) -> &HashMap<String, Arc<RwLock<ModelInstanciation>>> {
        &self.model_instanciations
    }

    pub fn treatments(&self) -> &HashMap<String, Arc<RwLock<Treatment>>> {
        &self.treatments
    }

    pub fn connections(&self) -> &Vec<Arc<RwLock<Connection>>> {
        &self.connections
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        
        // TODO Maybe should we check if no circular
        // references in connections there

        // Counting number of outputs connected to self outputs.
        let mut outputs_satisfaction = self.descriptor.outputs().iter().map(
                |(name, _output)| -> (String, usize) { (name.to_string(), 0) }
            ).collect::<HashMap<String, usize>>();

        for connection in &self.connections {

            let borrowed_connection = connection.read().unwrap();
            match borrowed_connection.output_treatment().as_ref().unwrap() {
                IO::Sequence() => {
                    *(outputs_satisfaction.get_mut(borrowed_connection.output_name().as_ref().unwrap()).unwrap()) += 1;
                }
                _ => {}
            }
        }
        
        // Check self outputs are connected to exactly one treatment output.
        for (_output, count) in outputs_satisfaction {

            if count < 1 {
                return Err(LogicError::unsatisfied_output())
            }
            else if count > 1 {
                return Err(LogicError::overloaded_output())
            }
        }

        Ok(())
    }

    pub fn register(&self) -> Result<(), LogicError> {

        self.validate()?;

        self.descriptor.register_builder(Box::new(SequenceBuilder::new(&self.auto_reference.upgrade().unwrap())));

        Ok(())
    }
}

