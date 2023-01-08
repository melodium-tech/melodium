
use core::fmt::{Debug};
use melodium_common::descriptor::{Collection, Parameterized, Model as ModelDescriptor, Parameter as ParameterDescriptor, Treatment as TreatmentDescriptor, Variability};
use super::{Treatment, Parameter, Value, Scope, Connection, IO};
use crate::descriptor;
use crate::design::{Model as ModelDesign, Parameter as ParameterDesign};
use crate::error::LogicError;
use std::sync::{Arc, RwLock, Weak};
use std::collections::{HashMap};

#[derive(Debug)]
pub struct TreatmentInstanciation {

    host_treatment: Weak<RwLock<Treatment>>,
    descriptor: Weak<dyn TreatmentDescriptor>,
    name: String,
    models: HashMap<String, String>,
    parameters: HashMap<String, Arc<RwLock<Parameter>>>,

    auto_reference: Weak<RwLock<Self>>,
}

impl TreatmentInstanciation {
    pub fn new(host_treatment: &Arc<RwLock<Treatment>>, descriptor: &Arc<dyn TreatmentDescriptor>, name: &str) -> Arc<RwLock<Self>> {
        Arc::<RwLock<Self>>::new_cyclic(|me| RwLock::new(Self {
            host_treatment: Arc::downgrade(host_treatment),
            descriptor: Arc::downgrade(descriptor),
            name: name.to_string(),
            models: HashMap::with_capacity(descriptor.models().len()),
            parameters: HashMap::with_capacity(descriptor.parameters().len()),
            auto_reference: me.clone(),
        }))
    }

    pub fn descriptor(&self) -> Arc<dyn TreatmentDescriptor> {
        self.descriptor.upgrade().unwrap()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_model(&mut self, parametric_name: &str, local_name: &str) -> Result<(), LogicError> {

        if self.descriptor().models().contains_key(parametric_name) {

            let rc_host = self.host_treatment.upgrade().unwrap();
            let borrowed_host = rc_host.read().unwrap();

            let mut base_model_descriptor = None;
            if let Some(model_descriptor) = borrowed_host.descriptor().models().get(local_name) {
                base_model_descriptor = Some(model_descriptor.base_model().unwrap_or_else(|| Arc::clone(&model_descriptor)));
            }
            else if let Some(model_instanciation) = borrowed_host.model_instanciations().get(local_name) {
                let model_instanciation_descriptor = model_instanciation.read().unwrap().descriptor();
                base_model_descriptor = Some(model_instanciation_descriptor.base_model().unwrap_or_else(|| Arc::clone(&model_instanciation_descriptor)));
            }

            if let Some(model_descriptor) = base_model_descriptor {

                if Arc::ptr_eq(&model_descriptor, self.descriptor().models().get(parametric_name).unwrap()) {
                    self.models.insert(parametric_name.to_string(), local_name.to_string());

                    Ok(())
                }
                else {
                    Err(LogicError::unmatching_model_type())
                }
            }
            else {
                Err(LogicError::unexisting_model())
            }
        }
        else {
            Err(LogicError::unexisting_parametric_model())
        }

    }

    pub fn add_parameter(&mut self, name: &str) -> Result<Arc<RwLock<Parameter>>, LogicError> {
        
        if self.descriptor().parameters().contains_key(name) {
            let parameter = Parameter::new( &(self.host_treatment.upgrade().unwrap() as Arc<RwLock<dyn Scope>>), 
                                            &self.descriptor().as_parameterized(),
                                            name
                                        );
            let rc_parameter = Arc::new(RwLock::new(parameter));

            if self.parameters.insert(name.to_string(), Arc::clone(&rc_parameter)).is_none() {
                Ok(rc_parameter)
            }
            else {
                Err(LogicError::multiple_parameter_assignation())
            }
        }
        else {
            Err(LogicError::unexisting_parameter())
        }
    }

    pub fn models(&self) -> &HashMap<String, String> {
        &self.models
    }

    pub fn parameters(&self) -> &HashMap<String, Arc<RwLock<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> Result<(), LogicError> {

        for (_, param) in &self.parameters {
            param.read().unwrap().validate()?;
        }

        let descriptor = self.descriptor();
        
        // Check if all parameters are filled.
        let unset_params: Vec<&ParameterDescriptor> = descriptor.parameters().iter().filter_map(
            |(core_param_name, core_param)|
            if self.parameters.contains_key(core_param_name) {
                None
            }
            else if core_param.default().is_some() {
                None
            }
            else {
                Some(core_param)
            }
        ).collect();

        if !unset_params.is_empty() {
            return Err(LogicError::unset_parameter());
        }

        // Check if all models are filled
        let unset_models: Vec<&String> = descriptor.models().iter().filter_map(
            |(model_name, _)|
            if self.models.contains_key(model_name) {
                None
            }
            else {
                Some(model_name)
            }
        ).collect();

        if !unset_models.is_empty() {
            return Err(LogicError::unset_model());
        }

        // Check if context values refers to available context.
        let rc_host = self.host_treatment.upgrade().unwrap();
        let borrowed_host = rc_host.read().unwrap();
        if let Some(_unavailable_context) = self.parameters.iter().find(
            |(_param_name, param)|
            match param.read().unwrap().value().as_ref().unwrap() {
                Value::Context(context, _entry) => 
                    !borrowed_host.descriptor().contexts().values().any(|c| Arc::ptr_eq(c, context)),
                _ => false
            }
        ) {
            return Err(LogicError::unavailable_context())
        }

        Ok(())
    }

    pub fn level(&self) -> usize {

        let rc_host = self.host_treatment.upgrade().unwrap();
        let borrowed_host = rc_host.read().unwrap();
        let all_connections = borrowed_host.connections();

        // We initialize the considered connection by taking only the ones were the current treatment
        // is set as input (end point of the connection).
        let mut considered_connections: Vec<&Connection> = all_connections.iter().filter_map(
            |raw_conn|
            match &raw_conn.input_treatment {
                IO::Sequence() => None,
                IO::Treatment(t) => {
                    // We want the input (end point) to be the current treatment, and the output (start point) to not be 'Self'-sequence.
                    if self.auto_reference.ptr_eq(&t) && raw_conn.output_treatment != IO::Sequence() {
                        Some(raw_conn)
                    }
                    else {
                        None
                    }
                }
            }
        ).collect();

        let mut level = 0;

        while considered_connections.len() > 0 {

            level += 1;

            // We retain only connections that have as input (end point) a treatment which is an ancestor
            // of the current treatment (output, start point).
            let next_considered_connections = all_connections.iter().filter_map(
                |raw_conn|

                if considered_connections.iter().any(
                    |conn|
                    conn.output_treatment == raw_conn.input_treatment
                ) {
                    Some(raw_conn)
                }
                else {
                    None
                }
            ).collect();

            considered_connections = next_considered_connections;
        }

        level
    }
}
