use super::{Connection, Parameter, Reference, Scope, Treatment, Value, IO, Generic};
use crate::design::TreatmentInstanciation as TreatmentInstanciationDesign;
use crate::error::{LogicError, LogicResult};
use core::fmt::Debug;
use melodium_common::descriptor::{
    Attribuable, Attribute, Attributes, Collection, Identified, Identifier,
    Parameter as ParameterDescriptor, Treatment as TreatmentDescriptor, DataType, DescribedType,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct TreatmentInstanciation {
    host_descriptor: Weak<dyn TreatmentDescriptor>,
    host_treatment: Weak<RwLock<Treatment>>,
    host_id: Identifier,
    descriptor: Weak<dyn TreatmentDescriptor>,
    name: String,
    generics: HashMap<String, DescribedType>,
    models: HashMap<String, String>,
    parameters: HashMap<String, Arc<RwLock<Parameter>>>,
    attributes: Attributes,

    design_reference: Option<Arc<dyn Reference>>,

    auto_reference: Weak<RwLock<Self>>,
}

impl TreatmentInstanciation {
    pub fn new(
        host_descriptor: &Arc<dyn TreatmentDescriptor>,
        host_treatment: &Arc<RwLock<Treatment>>,
        host_id: Identifier,
        descriptor: &Arc<dyn TreatmentDescriptor>,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Arc<RwLock<Self>> {
        Arc::<RwLock<Self>>::new_cyclic(|me| {
            RwLock::new(Self {
                host_descriptor: Arc::downgrade(host_descriptor),
                host_treatment: Arc::downgrade(host_treatment),
                host_id,
                descriptor: Arc::downgrade(descriptor),
                name: name.to_string(),
                generics: HashMap::with_capacity(descriptor.generics().len()),
                models: HashMap::with_capacity(descriptor.models().len()),
                parameters: HashMap::with_capacity(descriptor.parameters().len()),
                attributes: Attributes::default(),
                design_reference,
                auto_reference: me.clone(),
            })
        })
    }

    pub fn descriptor(&self) -> Arc<dyn TreatmentDescriptor> {
        self.descriptor.upgrade().unwrap()
    }

    pub fn design_reference(&self) -> &Option<Arc<dyn Reference>> {
        &self.design_reference
    }

    pub(crate) fn import_design(
        &mut self,
        design: &TreatmentInstanciationDesign,
        collection: &Arc<Collection>,
        replace: &HashMap<Identifier, Identifier>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        for (name, parameter_design) in &design.parameters {
            if let Some(parameter) = result
                .merge_degrade_failure(self.add_parameter(name, self.design_reference.clone()))
            {
                result.merge_degrade_failure(parameter.write().unwrap().import_design(
                    parameter_design,
                    collection,
                    replace,
                ));
            }
        }

        for (parametric_name, local_name) in &design.models {
            result.merge_degrade_failure(self.add_model(parametric_name, local_name));
        }

        result
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn add_attribute(&mut self, name: String, attribute: Attribute) {
        self.attributes.insert(name, attribute);
    }

    pub fn remove_attribute(&mut self, name: &str) -> bool {
        match self.attributes.remove(name) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn add_model(&mut self, parametric_name: &str, local_name: &str) -> LogicResult<()> {
        self.models
            .insert(parametric_name.to_string(), local_name.to_string());

        LogicResult::new_success(())
    }

    fn check_model(
        &self,
        parent: &Treatment,
        parametric_name: &str,
        local_name: &str,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        if self.descriptor().models().contains_key(parametric_name) {
            let model_descriptor = if let Some(model_descriptor) =
                parent.descriptor().models().get(local_name)
            {
                Some(Arc::clone(model_descriptor))
            } else if let Some(model_instanciation) = parent.model_instanciations().get(local_name)
            {
                Some(model_instanciation.read().unwrap().descriptor())
            } else {
                None
            };

            if let Some(mut parent_model_descriptor) = model_descriptor.clone() {
                let looking_for =
                    Arc::clone(self.descriptor().models().get(parametric_name).unwrap());
                let is_matching = loop {
                    if looking_for.identifier() == parent_model_descriptor.identifier() {
                        break true;
                    } else if let Some(base) = parent_model_descriptor.base_model() {
                        parent_model_descriptor = base;
                    } else {
                        break false;
                    }
                };

                if !is_matching {
                    result.errors_mut().push(LogicError::unmatching_model_type(
                        53,
                        parent.identifier().clone(),
                        self.descriptor().identifier().clone(),
                        parametric_name.to_string(),
                        looking_for.identifier().clone(),
                        local_name.to_string(),
                        model_descriptor.unwrap().identifier().clone(),
                        self.design_reference.clone(),
                    ));
                }
            } else {
                result.errors_mut().push(LogicError::undeclared_model(
                    42,
                    parent.identifier().clone(),
                    local_name.to_string(),
                    self.design_reference.clone(),
                ));
            }
        } else {
            result
                .errors_mut()
                .push(LogicError::unexisting_parametric_model(
                    54,
                    parent.identifier().clone(),
                    self.descriptor().identifier().clone(),
                    parametric_name.to_string(),
                    self.design_reference.clone(),
                ))
        }

        result
    }

    pub fn remove_model(&mut self, parametric_name: &str) -> LogicResult<bool> {
        Ok(if let Some(_) = self.models.remove(parametric_name) {
            true
        } else {
            false
        })
        .into()
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
                    26,
                    self.host_id.clone(),
                    self.descriptor().identifier().clone(),
                    self.name.clone(),
                    design_reference.clone(),
                ),
            ));
        }

        if !self.descriptor().parameters().contains_key(name) {
            result.errors_mut().push(LogicError::unexisting_parameter(
                11,
                self.host_id.clone(),
                self.descriptor().identifier().clone(),
                self.name.clone(),
                design_reference.clone(),
            ));
        }

        result.and(Ok(rc_parameter).into())
    }

    pub fn remove_parameter(&mut self, name: &str) -> LogicResult<bool> {
        Ok(if let Some(_) = self.parameters.remove(name) {
            true
        } else {
            false
        })
        .into()
    }

    pub fn models(&self) -> &HashMap<String, String> {
        &self.models
    }

    pub fn parameters(&self) -> &HashMap<String, Arc<RwLock<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());
        let rc_host = self.host_treatment.upgrade().unwrap();
        let borrowed_host = rc_host.read().unwrap();

        let descriptor = self.descriptor();

        result = self
            .parameters
            .iter()
            .fold(result, |mut result, (name, param)| {
                if !self.descriptor().parameters().contains_key(name) {
                    result.errors_mut().push(LogicError::unexisting_parameter(
                        203,
                        borrowed_host.identifier().clone(),
                        descriptor.identifier().clone(),
                        self.name.clone(),
                        self.design_reference.clone(),
                    ));
                }

                result.and_degrade_failure(param.read().unwrap().validate())
            });

        // Check if all parameters are filled.
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
                23,
                borrowed_host.descriptor().identifier().clone(),
                descriptor.identifier().clone(),
                unset_param.name().to_string(),
                self.design_reference.clone(),
            ));
        }

        for (param_name, local_name) in &self.models {
            result = result.and_degrade_failure(self.check_model(
                &borrowed_host,
                param_name,
                local_name,
            ));
        }

        // Check if all models are filled
        let unset_models: Vec<&String> = descriptor
            .models()
            .iter()
            .filter_map(|(model_name, _)| {
                if self.models.contains_key(model_name) {
                    None
                } else {
                    Some(model_name)
                }
            })
            .collect();

        for unset_model in unset_models {
            result.errors_mut().push(LogicError::unset_model(
                55,
                borrowed_host.descriptor().identifier().clone(),
                descriptor.identifier().clone(),
                unset_model.clone(),
                self.design_reference.clone(),
            ));
        }

        // Check if context values refers to available context.
        for (_param_name, param) in self.parameters.iter() {
            match param.read().unwrap().value().as_ref() {
                Some(Value::Context(context, _entry)) => {
                    if !borrowed_host
                        .descriptor()
                        .contexts()
                        .values()
                        .any(|c| Arc::ptr_eq(c, context))
                    {
                        result.errors_mut().push(LogicError::unavailable_context(
                            30,
                            borrowed_host.descriptor().identifier().clone(),
                            context.identifier().clone(),
                            param.read().unwrap().design_reference().clone(),
                        ));
                    }
                }
                _ => {}
            }
        }

        result
    }

    pub fn level(&self) -> usize {
        let rc_host = self.host_treatment.upgrade().unwrap();
        let borrowed_host = rc_host.read().unwrap();
        let all_connections = borrowed_host.connections();

        // We initialize the considered connection by taking only the ones were the current treatment
        // is set as input (end point of the connection).
        let mut considered_connections: Vec<&Connection> = all_connections
            .iter()
            .filter_map(|raw_conn| match &raw_conn.input_treatment {
                IO::Sequence() => None,
                IO::Treatment(t) => {
                    // We want the input (end point) to be the current treatment, and the output (start point) to not be 'Self'-sequence.
                    if self.auto_reference.ptr_eq(&t) && raw_conn.output_treatment != IO::Sequence()
                    {
                        Some(raw_conn)
                    } else {
                        None
                    }
                }
            })
            .collect();

        let mut level = 0;

        while considered_connections.len() > 0 {
            level += 1;

            // We retain only connections that have as input (end point) a treatment which is an ancestor
            // of the current treatment (output, start point).
            let next_considered_connections = all_connections
                .iter()
                .filter_map(|raw_conn| {
                    if considered_connections
                        .iter()
                        .any(|conn| conn.output_treatment == raw_conn.input_treatment)
                    {
                        Some(raw_conn)
                    } else {
                        None
                    }
                })
                .collect();

            considered_connections = next_considered_connections;
        }

        level
    }
}

impl Attribuable for TreatmentInstanciation {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl Generic for TreatmentInstanciation {
    fn generics(&self) -> &HashMap<String, DescribedType> {
        &self.generics
    }
}
