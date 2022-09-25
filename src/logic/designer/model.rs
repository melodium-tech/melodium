
use std::sync::{Arc, Weak, RwLock};
use std::collections::HashMap;
use super::super::error::LogicError;
use super::super::collection_pool::CollectionPool;
use super::super::descriptor::ConfiguredModelDescriptor;
use super::super::descriptor::ModelDescriptor;
use super::super::descriptor::DesignableDescriptor;
use super::super::descriptor::ParameterizedDescriptor;
use super::super::descriptor::ParameterDescriptor;
use super::scope::Scope;
use super::parameter::Parameter;
use super::value::Value;

use super::super::builder::configured_model_builder::ConfiguredModelBuilder;

#[derive(Debug)]
pub struct Model {
    collections: Arc<CollectionPool>,
    descriptor: Arc<ConfiguredModelDescriptor>,

    parameters: HashMap<String, Arc<RwLock<Parameter>>>,

    auto_reference: Weak<RwLock<Self>>,
}

impl Model {

    pub fn new(collections: &Arc<CollectionPool>, descriptor: &Arc<ConfiguredModelDescriptor>) -> Arc<RwLock<Self>> {
        
        let this = Arc::<RwLock<Self>>::new_cyclic(|me| RwLock::new(Self {
            collections: Arc::clone(collections),
            descriptor: Arc::clone(descriptor),
            parameters: HashMap::new(),
            auto_reference: me.clone(),
        }));

        descriptor.set_designer(Arc::clone(&this));

        this
    }

    pub fn collections(&self) -> &Arc<CollectionPool> {
        &self.collections
    }

    pub fn descriptor(&self) -> &Arc<ConfiguredModelDescriptor> {
        &self.descriptor
    }

    pub fn add_parameter(&mut self, name: &str) -> Result<Arc<RwLock<Parameter>>, LogicError> {
        
        if self.descriptor.core_model().parameters().contains_key(name) {
            let parameter = Parameter::new( &(self.auto_reference.upgrade().unwrap() as Arc<RwLock<dyn Scope>>),
                                            &(Arc::clone(&self.descriptor.core_model()) as Arc<dyn ParameterizedDescriptor>),
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

    pub fn parameters(&self) -> &HashMap<String, Arc<RwLock<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> Result<(), LogicError> {

        for (_, param) in &self.parameters {
            param.read().unwrap().validate()?;
        }

        // Check if all parent parameters are filled.
        let rc_core_model = self.descriptor.core_model();
        let unset_params: Vec<&ParameterDescriptor> = rc_core_model.parameters().iter().filter_map(
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

        // Check all parameters does not refers to a context.
        if let Some(_forbidden_context) = self.parameters.iter().find(|&(_param_name, param)| matches!(param.read().unwrap().value(), Some(Value::Context{..}))) {
            return Err(LogicError::no_context())
        }

        Ok(())
    }

    pub fn register(&self) -> Result<(), LogicError> {
        
        self.validate()?;

        self.descriptor.register_builder(Box::new(ConfiguredModelBuilder::new(&self.auto_reference.upgrade().unwrap())));

        Ok(())
    }
}

impl Scope for Model {

    fn descriptor(&self) -> Arc<dyn ParameterizedDescriptor> {
        Arc::clone(&self.descriptor) as Arc<dyn ParameterizedDescriptor>
    }

    fn collections(&self) -> Arc<CollectionPool> {
        Arc::clone(&self.collections)
    }
}
