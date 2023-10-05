use super::{Parameter, Reference, Scope, Value};
use crate::descriptor::Model as ModelDescriptor;
use crate::design::{Model as ModelDesign, Parameter as ParameterDesign};
use crate::error::{LogicError, LogicResult};
use core::fmt::Debug;
use melodium_common::descriptor::{
    Collection, Identified, Identifier, Model as ModelTrait, Parameter as ParameterDescriptor,
    Parameterized,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct Model {
    collection: Arc<Collection>,
    descriptor: Weak<ModelDescriptor>,

    parameters: HashMap<String, Arc<RwLock<Parameter>>>,

    design_reference: Option<Arc<dyn Reference>>,

    auto_reference: Weak<RwLock<Self>>,
}

impl Model {
    pub fn new(
        descriptor: &Arc<ModelDescriptor>,
        collection: Arc<Collection>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Arc<RwLock<Self>> {
        Arc::<RwLock<Self>>::new_cyclic(|me| {
            RwLock::new(Self {
                descriptor: Arc::downgrade(descriptor),
                collection,
                parameters: HashMap::new(),
                design_reference,
                auto_reference: me.clone(),
            })
        })
    }

    pub fn collection(&self) -> &Arc<Collection> {
        &self.collection
    }

    pub fn descriptor(&self) -> Arc<ModelDescriptor> {
        self.descriptor.upgrade().unwrap()
    }

    pub fn design_reference(&self) -> &Option<Arc<dyn Reference>> {
        &self.design_reference
    }

    pub fn import_design(
        &mut self,
        design: &ModelDesign,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        for (name, parameter_design) in &design.parameters {
            if let Some(parameter) =
                result.merge_degrade_failure(self.add_parameter(name, design_reference.clone()))
            {
                result.merge_degrade_failure(
                    parameter
                        .write()
                        .unwrap()
                        .import_design(parameter_design, &self.collection),
                );
            }
        }

        result
    }

    pub fn add_parameter(
        &mut self,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<Arc<RwLock<Parameter>>> {
        let base_model = self
            .descriptor()
            .base_model()
            .expect("Designed model must have base model");

        if base_model.parameters().contains_key(name) {
            let descriptor = self.descriptor();
            let parameter = Parameter::new(
                &(self.auto_reference.upgrade().unwrap() as Arc<RwLock<dyn Scope>>),
                &(descriptor.clone() as Arc<dyn Parameterized>),
                descriptor.identifier().clone(),
                &base_model.as_parameterized(),
                name,
                design_reference.clone(),
            );
            let rc_parameter = Arc::new(RwLock::new(parameter));

            if self
                .parameters
                .insert(name.to_string(), Arc::clone(&rc_parameter))
                .is_none()
            {
                Ok(rc_parameter).into()
            } else {
                Err(LogicError::multiple_parameter_assignation(
                    25,
                    self.descriptor().identifier().clone(),
                    base_model.identifier().clone(),
                    name.to_string(),
                    design_reference,
                )
                .into())
                .into()
            }
        } else {
            Err(LogicError::unexisting_parameter(
                12,
                self.descriptor().identifier().clone(),
                base_model.identifier().clone(),
                name.to_string(),
                design_reference,
            )
            .into())
            .into()
        }
    }

    pub fn remove_parameter(&mut self, name: &str) -> LogicResult<bool> {
        Ok(match self.parameters.remove(name) {
            Some(_) => true,
            None => false,
        })
        .into()
    }

    pub fn parameters(&self) -> &HashMap<String, Arc<RwLock<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        result = self.parameters.iter().fold(result, |result, (_, param)| {
            result.and_degrade_failure(param.read().unwrap().validate())
        });

        // Check if all parent parameters are filled.
        let rc_base_model = self
            .descriptor()
            .base_model()
            .expect("Designed model must have base model");
        let unset_params: Vec<&ParameterDescriptor> = rc_base_model
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
                21,
                self.descriptor().identifier().clone(),
                rc_base_model.identifier().clone(),
                unset_param.name().to_string(),
                self.design_reference.clone(),
            ));
        }

        // Check all parameters does not refers to a context.
        for (name, param) in self.parameters.iter().filter(|&(_param_name, param)| {
            matches!(param.read().unwrap().value(), Some(Value::Context { .. }))
        }) {
            result.errors_mut().push(LogicError::no_context(
                29,
                self.descriptor().identifier().clone(),
                rc_base_model.identifier().clone(),
                rc_base_model.identifier().name().to_string(),
                name.to_string(),
                param.read().unwrap().design_reference().clone(),
            ));
        }

        result
    }

    pub fn make_use(&self, identifier: &Identifier) -> bool {
        self.unvalidated_design()
            .success()
            .map(|design| design.make_use(identifier))
            .unwrap_or(false)
    }

    pub fn unvalidated_design(&self) -> LogicResult<ModelDesign> {
        let result = LogicResult::new_success(());

        result.and_then(|_| {
            LogicResult::new_success(ModelDesign {
                descriptor: self.descriptor.clone(),
                parameters: self
                    .parameters
                    .iter()
                    .filter_map(|(name, param)| {
                        if let Some(value) = param.read().unwrap().value().as_ref() {
                            Some((
                                name.clone(),
                                ParameterDesign {
                                    name: name.clone(),
                                    value: value.clone(),
                                },
                            ))
                        } else {
                            None
                        }
                    })
                    .collect(),
            })
        })
    }

    pub fn design(&self) -> LogicResult<ModelDesign> {
        let result = self.validate();

        result.and_then(|_| {
            LogicResult::new_success(ModelDesign {
                descriptor: self.descriptor.clone(),
                parameters: self
                    .parameters
                    .iter()
                    .map(|(name, param)| {
                        (
                            name.clone(),
                            ParameterDesign {
                                name: name.clone(),
                                value: param.read().unwrap().value().as_ref().unwrap().clone(),
                            },
                        )
                    })
                    .collect(),
            })
        })
    }
}

impl Scope for Model {
    fn descriptor(&self) -> Arc<dyn Parameterized> {
        Arc::clone(&self.descriptor()) as Arc<dyn Parameterized>
    }

    fn collection(&self) -> Arc<Collection> {
        Arc::clone(&self.collection)
    }

    fn identifier(&self) -> Identifier {
        self.descriptor().identifier().clone()
    }
}
