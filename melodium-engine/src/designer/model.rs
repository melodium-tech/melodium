use super::{Parameter, Scope, Value};
use crate::descriptor::Model as ModelDescriptor;
use crate::design::{Model as ModelDesign, Parameter as ParameterDesign};
use crate::error::LogicError;
use core::fmt::Debug;
use melodium_common::descriptor::{
    Collection, Model as ModelTrait, Parameter as ParameterDescriptor, Parameterized,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct Model {
    collection: Option<Arc<Collection>>,
    descriptor: Weak<ModelDescriptor>,

    parameters: HashMap<String, Arc<RwLock<Parameter>>>,

    auto_reference: Weak<RwLock<Self>>,
}

impl Model {
    pub fn new(descriptor: &Arc<ModelDescriptor>) -> Arc<RwLock<Self>> {
        Arc::<RwLock<Self>>::new_cyclic(|me| {
            RwLock::new(Self {
                descriptor: Arc::downgrade(descriptor),
                collection: None,
                parameters: HashMap::new(),
                auto_reference: me.clone(),
            })
        })
    }

    pub fn set_collection(&mut self, collection: Arc<Collection>) {
        self.collection = Some(collection);
    }

    pub fn collection(&self) -> &Option<Arc<Collection>> {
        &self.collection
    }

    pub fn descriptor(&self) -> Arc<ModelDescriptor> {
        self.descriptor.upgrade().unwrap()
    }

    pub fn add_parameter(&mut self, name: &str) -> Result<Arc<RwLock<Parameter>>, LogicError> {
        if self
            .descriptor()
            .base_model()
            .expect("Designed model must have base model")
            .parameters()
            .contains_key(name)
        {
            let parameter = Parameter::new(
                &(self.auto_reference.upgrade().unwrap() as Arc<RwLock<dyn Scope>>),
                &self
                    .descriptor()
                    .base_model()
                    .expect("Designed model must have base model")
                    .as_parameterized(),
                name,
            );
            let rc_parameter = Arc::new(RwLock::new(parameter));

            if self
                .parameters
                .insert(name.to_string(), Arc::clone(&rc_parameter))
                .is_none()
            {
                Ok(rc_parameter)
            } else {
                Err(LogicError::multiple_parameter_assignation())
            }
        } else {
            Err(LogicError::unexisting_parameter())
        }
    }

    pub fn parameters(&self) -> &HashMap<String, Arc<RwLock<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        for (_, param) in &self.parameters {
            param.read().unwrap().validate()?;
        }

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

        if !unset_params.is_empty() {
            return Err(LogicError::unset_parameter());
        }

        // Check all parameters does not refers to a context.
        if let Some(_forbidden_context) = self.parameters.iter().find(|&(_param_name, param)| {
            matches!(param.read().unwrap().value(), Some(Value::Context { .. }))
        }) {
            return Err(LogicError::no_context());
        }

        Ok(())
    }

    pub fn design(&self) -> Result<ModelDesign, LogicError> {
        self.validate()?;

        Ok(ModelDesign {
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
    }
}

impl Scope for Model {
    fn descriptor(&self) -> Arc<dyn Parameterized> {
        Arc::clone(&self.descriptor()) as Arc<dyn Parameterized>
    }

    fn collection(&self) -> Option<Arc<Collection>> {
        self.collection
            .as_ref()
            .map(|collection| Arc::clone(collection))
    }
}
