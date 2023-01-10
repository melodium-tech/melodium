
use core::fmt::{Debug};
use melodium_common::descriptor::{Identifier, Collection, Entry, Parameterized, Model as ModelTrait, Parameter as ParameterDescriptor, Treatment as TreatmentTrait};
use super::{Scope, Parameter, Value, ModelInstanciation, TreatmentInstanciation, Connection, IO};
use crate::descriptor::Treatment as TreatmentDescriptor;
use crate::design::{Treatment as TreatmentDesign, Connection as ConnectionDesign, IO as IODesign, Parameter as ParameterDesign, ModelInstanciation as ModelInstanciationDesign, TreatmentInstanciation as TreatmentInstanciationDesign, treatment_instanciation};
use crate::error::LogicError;
use std::sync::{Arc, RwLock, Weak};
use std::collections::{HashMap};

#[derive(Debug)]
pub struct Treatment {
    collection: Option<Arc<Collection>>,
    descriptor: Weak<TreatmentDescriptor>,

    model_instanciations: HashMap<String, Arc<RwLock<ModelInstanciation>>>,
    treatments: HashMap<String, Arc<RwLock<TreatmentInstanciation>>>,
    connections: Vec<Connection>,

    auto_reference: Weak<RwLock<Self>>,
}

impl Treatment {
    pub fn new(descriptor: &Arc<TreatmentDescriptor>) -> Arc<RwLock<Self>> {

        Arc::<RwLock<Self>>::new_cyclic(|me| RwLock::new(Self {
            descriptor: Arc::downgrade(descriptor),
            collection: None,
            model_instanciations: HashMap::new(),
            treatments: HashMap::new(),
            connections: Vec::new(),
            auto_reference: me.clone(),
        }))
    }

    pub fn set_collection(&mut self, collection: std::sync::Arc<melodium_common::descriptor::Collection>) {
        self.collection = Some(collection);
    }

    pub fn collection(&self) -> &Option<Arc<Collection>> {
        &self.collection
    }

    pub fn descriptor(&self) -> Arc<TreatmentDescriptor> {
        self.descriptor.upgrade().unwrap()
    }

    pub fn add_model_instanciation(&mut self, model_identifier: &Identifier, name: &str) -> Result<Arc<RwLock<ModelInstanciation>>, LogicError> {

        if let Some(Entry::Model(model_descriptor)) = self.collection.as_ref().ok_or_else(|| LogicError::collection_undefined())?.get(model_identifier) {
    
            let model = ModelInstanciation::new(&self.auto_reference.upgrade().unwrap(), model_descriptor, name);
            let rc_model = Arc::new(RwLock::new(model));
            self.model_instanciations.insert(name.to_string(), Arc::clone(&rc_model));
            Ok(rc_model)
        }
        else {
            Err(LogicError::unexisting_model())
        }
    }

    pub fn remove_model_instanciation(&mut self, name: &str) -> Result<bool, LogicError> {

        if let Some(_) = self.model_instanciations.remove(name) {
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    pub fn add_treatment(&mut self, treatment_identifier: &Identifier, name: &str) -> Result<Arc<RwLock<TreatmentInstanciation>>, LogicError> {
        
        if let Some(Entry::Treatment(treatment_descriptor)) = self.collection.as_ref().ok_or_else(|| LogicError::collection_undefined())?.get(treatment_identifier) {
            let rc_treatment = TreatmentInstanciation::new(&self.auto_reference.upgrade().unwrap(), treatment_descriptor, name);
            self.treatments.insert(name.to_string(), Arc::clone(&rc_treatment));
            Ok(rc_treatment)
        }
        else {
            Err(LogicError::unexisting_treatment())
        }

    }

    pub fn remove_treatment(&mut self, name: &str) -> Result<bool, LogicError> {

        if let Some(ref treatment) = self.treatments.remove(name) {
            self.connections.retain(|conn| {
                !
                if let IO::Treatment(input_treatment) = &conn.input_treatment {
                    input_treatment.ptr_eq(&Arc::downgrade(treatment))
                }
                else if let IO::Treatment(output_treatment) = &conn.output_treatment {
                    output_treatment.ptr_eq(&Arc::downgrade(treatment))
                }
                else {
                    false
                }
            });

            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    pub fn add_connection(&mut self, output_treatment: &str, output_name: &str, input_treatment: &str, input_name: &str) -> Result<(), LogicError> {
        
        let rc_output_treatment;
        let output;
        if let Some(pos_rc_output_treatment) = self.treatments.get(output_treatment) {
            rc_output_treatment = pos_rc_output_treatment;

            if let Some(pos_output) = rc_output_treatment.read().unwrap().descriptor().outputs().get(output_name) {
                output = pos_output.clone();
            }
            else {
                return Err(LogicError::connection_output_not_found())
            }
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        let rc_input_treatment;
        let input;
        if let Some(pos_rc_input_treatment) = self.treatments.get(input_treatment) {
            rc_input_treatment = pos_rc_input_treatment;

            if let Some(pos_input) = rc_input_treatment.read().unwrap().descriptor().inputs().get(input_name) {
                input = pos_input.clone();
            }
            else {
                return Err(LogicError::connection_input_not_found())
            }
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        if input.matches_output(&output) {
            self.connections.push(Connection::new_internal(output_name, rc_output_treatment, input_name, rc_input_treatment));
            Ok(())
        }
        else {
            return Err(LogicError::unexisting_connexion_type())
        }
    }

    pub fn remove_connection(&mut self, output_treatment: &str, output_name: &str, input_treatment: &str, input_name: &str) -> Result<bool, LogicError> {

        let mut found = false;
        self.connections.retain(|connection| {
            if connection.output_name == output_name
            && connection.input_name == input_name
            && match &connection.output_treatment {
                IO::Treatment(t) => t.upgrade().unwrap().read().unwrap().name() == output_treatment,
                _ => false
            }
            && match &connection.input_treatment {
                IO::Treatment(t) => t.upgrade().unwrap().read().unwrap().name() == input_treatment,
                _ => false
            } {
                found = true;
                false
            } else {
                true
            }
        });

        Ok(found)
    }

    pub fn add_self_connection(&mut self, self_input_name: &str, self_output_name: &str) -> Result<(), LogicError> {

        let input_self;
        if let Some(pos_input) = self.descriptor().inputs().get(self_input_name) {
            input_self = pos_input.clone();
        }
        else {
            return Err(LogicError::connection_self_input_not_found())
        }

        let output_self;
        if let Some(pos_output) = self.descriptor().outputs().get(self_output_name) {
            output_self = pos_output.clone();
        }
        else {
            return Err(LogicError::connection_self_output_not_found())
        }

        if input_self.matches_output(&output_self) {
            self.connections.push(Connection::new_self(input_self.name(), output_self.name()));
            Ok(())
        }
        else {
            return Err(LogicError::unexisting_connexion_type())
        }
    }

    pub fn remove_self_connection(&mut self, self_input_name: &str, self_output_name: &str) -> Result<bool, LogicError> {

        let mut found = false;
        self.connections.retain(|connection| {
            if connection.output_name == self_input_name
            && connection.input_name == self_output_name
            && match connection.output_treatment {
                IO::Sequence() => true,
                _ => false
            }
            && match connection.input_treatment {
                IO::Sequence() => true,
                _ => false
            } {
                found = true;
                false
            } else {
                true
            }
        });

        Ok(found)
    }

    pub fn add_input_connection(&mut self, self_input_name: &str, input_treatment: &str, input_name: &str) -> Result<(), LogicError> {

        let input_self;
        if let Some(pos_input) = self.descriptor().inputs().get(self_input_name) {
            input_self = pos_input.clone();
        }
        else {
            return Err(LogicError::connection_self_input_not_found())
        }

        let rc_input_treatment;
        let input;
        if let Some(pos_rc_input_treatment) = self.treatments.get(input_treatment) {
            rc_input_treatment = pos_rc_input_treatment;

            if let Some(pos_input) = rc_input_treatment.read().unwrap().descriptor().inputs().get(input_name) {
                input = pos_input.clone();
            }
            else {
                return Err(LogicError::connection_input_not_found())
            }
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        if input_self.matches_input(&input) {
            self.connections.push(Connection::new_self_to_internal(input_self.name(), input.name(), rc_input_treatment));
            Ok(())
        }
        else {
            return Err(LogicError::unexisting_connexion_type())
        }
    }

    pub fn remove_input_connection(&mut self, self_input_name: &str, input_treatment: &str, input_name: &str) -> Result<bool, LogicError> {

        let mut found = false;
        self.connections.retain(|connection| {
            if connection.output_name == self_input_name
            && connection.input_name == input_name
            && match connection.output_treatment {
                IO::Sequence() => true,
                _ => false
            }
            && match &connection.input_treatment {
                IO::Treatment(t) => t.upgrade().unwrap().read().unwrap().name() == input_treatment,
                _ => false
            } {
                found = true;
                false
            } else {
                true
            }
        });

        Ok(found)
    }

    pub fn add_output_connection(&mut self, self_output_name: &str, output_treatment: &str, output_name: &str) -> Result<(), LogicError> {

        let output_self;
        if let Some(pos_output) = self.descriptor().outputs().get(self_output_name) {
            output_self = pos_output.clone();
        }
        else {
            return Err(LogicError::connection_self_output_not_found())
        }

        let rc_output_treatment;
        let output;
        if let Some(pos_rc_output_treatment) = self.treatments.get(output_treatment) {
            rc_output_treatment = pos_rc_output_treatment;

            if let Some(pos_output) = rc_output_treatment.read().unwrap().descriptor().outputs().get(output_name) {
                output = pos_output.clone();
            }
            else {
                return Err(LogicError::connection_output_not_found())
            }
        }
        else {
            return Err(LogicError::undeclared_treatment())
        }

        if output_self.matches_output(&output) {
            self.connections.push(Connection::new_internal_to_self(output.name(), rc_output_treatment, output_self.name()));
            Ok(())
        }
        else {
            return Err(LogicError::unexisting_connexion_type())
        }
    }

    pub fn remove_output_connection(&mut self, self_output_name: &str, output_treatment: &str, output_name: &str) -> Result<bool, LogicError> {

        let mut found = false;
        self.connections.retain(|connection| {
            if connection.output_name == output_name
            && connection.input_name == self_output_name
            && match &connection.output_treatment {
                IO::Treatment(t) => t.upgrade().unwrap().read().unwrap().name() == output_treatment,
                _ => false
            }
            && match connection.input_treatment {
                IO::Sequence() => true,
                _ => false
            } {
                found = true;
                false
            } else {
                true
            }
        });

        Ok(found)
    }

    pub fn model_instanciations(&self) -> &HashMap<String, Arc<RwLock<ModelInstanciation>>> {
        &self.model_instanciations
    }

    pub fn treatments(&self) -> &HashMap<String, Arc<RwLock<TreatmentInstanciation>>> {
        &self.treatments
    }

    pub fn connections(&self) -> &Vec<Connection> {
        &self.connections
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        
        // TODO Maybe should we check if no circular
        // references in connections there

        // Counting number of outputs connected to self outputs.
        let mut outputs_satisfaction = self.descriptor().outputs().iter().map(
                |(name, _output)| -> (String, usize) { (name.to_string(), 0) }
            ).collect::<HashMap<String, usize>>();

        for connection in &self.connections {

            match connection.input_treatment {
                IO::Sequence() => {
                    *(outputs_satisfaction.get_mut(&connection.input_name).unwrap()) += 1;
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

    pub fn design(&self) -> Result<TreatmentDesign, LogicError> {
        
        self.validate()?;

        Ok(TreatmentDesign {
            descriptor: self.descriptor.clone(),
            model_instanciations: self.model_instanciations.iter().map(
                |(name, model_instanciation)| {
                    let model_instanciation = model_instanciation.read().unwrap();
                    (name.clone(), ModelInstanciationDesign {
                        name: name.clone(),
                        descriptor: Arc::downgrade(&model_instanciation.descriptor()),
                        parameters: model_instanciation.parameters().iter().map(
                            |(name, param)|
                                (name.clone(), ParameterDesign { name: name.clone(), value: param.read().unwrap().value().as_ref().unwrap().clone() })
                            ).collect()
                    })
                },
            ).collect(),
            treatments: self.treatments.iter().map(|(name, treatment_instanciation)| {
                let treatment_instanciation = treatment_instanciation.read().unwrap();
                (name.clone(), TreatmentInstanciationDesign {
                    name: name.clone(),
                    descriptor: Arc::downgrade(&treatment_instanciation.descriptor()),
                    models: treatment_instanciation.models().clone(),
                    parameters: treatment_instanciation.parameters().iter().map(
                        |(name, param)|
                            (name.clone(), ParameterDesign { name: name.clone(), value: param.read().unwrap().value().as_ref().unwrap().clone() })
                        ).collect()
                })
            }).collect(),
            connections: self.connections.iter().map(|connection| ConnectionDesign {
                output_treatment: match &connection.output_treatment {
                    IO::Sequence() => IODesign::Sequence(),
                    IO::Treatment(t) => IODesign::Treatment(t.upgrade().unwrap().read().unwrap().name().to_string())
                },
                output_name: connection.output_name.clone(),
                input_treatment: match &connection.input_treatment {
                    IO::Sequence() => IODesign::Sequence(),
                    IO::Treatment(t) => IODesign::Treatment(t.upgrade().unwrap().read().unwrap().name().to_string())
                },
                input_name: connection.input_name.clone(),
            }).collect(),
        })
    }
}

impl Scope for Treatment {

    fn descriptor(&self) -> Arc<dyn Parameterized> {
        Arc::clone(&self.descriptor()) as Arc<dyn Parameterized>
    }

    fn collection(&self) -> Option<Arc<Collection>> {
        self.collection.as_ref().map(|collection| Arc::clone(collection))
    }
}

