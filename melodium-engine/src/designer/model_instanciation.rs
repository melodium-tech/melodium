use super::{Parameter, Reference, Scope, Treatment, Value};
use crate::design::ModelInstanciation as ModelInstanciationDesign;
use crate::error::{LogicError, LogicResult};
use core::fmt::Debug;
use melodium_common::descriptor::{
    Collection, Identified, Identifier, Model as ModelDescriptor, Parameter as ParameterDescriptor,
    Treatment as TreatmentDescriptor, Variability,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct ModelInstanciation {
    host_descriptor: Weak<dyn TreatmentDescriptor>,
    host_treatment: Weak<RwLock<Treatment>>,
    host_id: Identifier,
    descriptor: Weak<dyn ModelDescriptor>,
    name: String,
    parameters: HashMap<String, Arc<RwLock<Parameter>>>,
    design_reference: Option<Arc<dyn Reference>>,
}

impl ModelInstanciation {
    pub fn new(
        host_descriptor: &Arc<dyn TreatmentDescriptor>,
        host_treatment: &Arc<RwLock<Treatment>>,
        host_id: Identifier,
        descriptor: &Arc<dyn ModelDescriptor>,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            host_descriptor: Arc::downgrade(host_descriptor),
            host_treatment: Arc::downgrade(host_treatment),
            host_id,
            descriptor: Arc::downgrade(descriptor),
            name: name.to_string(),
            parameters: HashMap::with_capacity(descriptor.parameters().len()),
            design_reference,
        }
    }

    pub fn descriptor(&self) -> Arc<dyn ModelDescriptor> {
        self.descriptor.upgrade().unwrap()
    }

    pub fn design_reference(&self) -> &Option<Arc<dyn Reference>> {
        &self.design_reference
    }

    pub(crate) fn import_design(
        &mut self,
        design: &ModelInstanciationDesign,
        collection: &Arc<Collection>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        for (name, parameter_design) in &design.parameters {
            if let Some(parameter) = result
                .merge_degrade_failure(self.add_parameter(name, self.design_reference.clone()))
            {
                result.merge_degrade_failure(
                    parameter
                        .write()
                        .unwrap()
                        .import_design(parameter_design, collection),
                );
            }
        }

        result
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_parameter(
        &mut self,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<Arc<RwLock<Parameter>>> {
        let mut result = LogicResult::new_success(());

        let host_descriptor = self.host_descriptor.upgrade().unwrap();
        let parameter = Parameter::new(
            &(self.host_treatment.upgrade().unwrap() as Arc<RwLock<dyn Scope>>),
            &host_descriptor.as_parameterized(),
            self.host_id.clone(),
            &self.descriptor().as_parameterized(),
            name,
            design_reference.clone(),
        );
        let rc_parameter = Arc::new(RwLock::new(parameter));

        if self
            .parameters
            .insert(name.to_string(), Arc::clone(&rc_parameter))
            .is_some()
        {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::multiple_parameter_assignation(
                    24,
                    self.host_id.clone(),
                    self.descriptor().identifier().clone(),
                    name.to_string(),
                    design_reference.clone(),
                ),
            ));
        }

        if !self.descriptor().parameters().contains_key(name) {
            result.errors_mut().push(LogicError::unexisting_parameter(
                10,
                self.host_id.clone(),
                self.descriptor().identifier().clone(),
                self.name.clone(),
                design_reference,
            ));
        }

        result.and(Ok(rc_parameter).into())
    }

    pub fn remove_parameter(&mut self, name: &str) -> LogicResult<bool> {
        if let Some(_) = self.parameters.remove(name) {
            Ok(true).into()
        } else {
            Ok(false).into()
        }
    }

    pub fn parameters(&self) -> &HashMap<String, Arc<RwLock<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        let rc_host = self.host_treatment.upgrade().unwrap();
        let host = rc_host.read().unwrap();
        let descriptor = self.descriptor();

        result = self
            .parameters
            .iter()
            .fold(result, |mut result, (name, param)| {
                if !self.descriptor().parameters().contains_key(name) {
                    result.errors_mut().push(LogicError::unexisting_parameter(
                        193,
                        host.identifier().clone(),
                        descriptor.identifier().clone(),
                        self.name.clone(),
                        self.design_reference.clone(),
                    ));
                }

                result.and_degrade_failure(param.read().unwrap().validate())
            });

        // Check if all model parameters are filled.
        let unset_params: Vec<&ParameterDescriptor> = descriptor
            .parameters()
            .iter()
            .filter_map(|(core_param_name, core_param)| {
                if self.parameters.contains_key(core_param_name) {
                    None
                } else if core_param.default().is_some() {
                    None
                } else {
                    Some(core_param)
                }
            })
            .collect();
        for unset_param in unset_params {
            result.errors_mut().push(LogicError::unset_parameter(
                22,
                host.descriptor().identifier().clone(),
                descriptor.identifier().clone(),
                unset_param.name().to_string(),
                self.design_reference.clone(),
            ));
        }

        // Check all parameters does not refers to a context.
        for (name, param) in self.parameters.iter().filter(|&(_param_name, param)| {
            matches!(param.read().unwrap().value(), Some(Value::Context { .. }))
        }) {
            result.errors_mut().push(LogicError::no_context(
                28,
                host.descriptor().identifier().clone(),
                descriptor.identifier().clone(),
                self.name.clone(),
                name.to_string(),
                param.read().unwrap().design_reference().clone(),
            ));
        }

        // Check all parameters are const.
        for (name, param) in self.parameters.iter().filter(|&(param_name, param)| {
            *param
                .read()
                .unwrap()
                .parent_descriptor()
                .upgrade()
                .unwrap()
                .parameters()
                .get(param_name)
                .unwrap()
                .variability()
                != Variability::Const
        }) {
            result
                .errors_mut()
                .push(LogicError::model_instanciation_const_only(
                    62,
                    host.descriptor().identifier().clone(),
                    descriptor.identifier().clone(),
                    self.name.clone(),
                    name.to_string(),
                    param.read().unwrap().design_reference().clone(),
                ));
        }

        result
    }
}
