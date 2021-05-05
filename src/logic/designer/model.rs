
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use super::super::error::LogicError;
use super::super::collection_pool::CollectionPool;
use super::super::descriptor::ConfiguredModelDescriptor;
use super::super::descriptor::ModelDescriptor;
use super::super::descriptor::DesignableDescriptor;
use super::super::descriptor::ParameterizedDescriptor;
use super::super::descriptor::ParameterDescriptor;
use super::parameter::Parameter;
use super::value::Value;

use super::super::builder::configured_model_builder::ConfiguredModelBuilder;

#[derive(Debug)]
pub struct Model {
    collections: Rc<CollectionPool>,
    descriptor: Rc<ConfiguredModelDescriptor>,

    parameters: HashMap<String, Rc<RefCell<Parameter>>>,

    auto_reference: Weak<RefCell<Self>>,
}

impl Model {

    pub fn new(collections: &Rc<CollectionPool>, descriptor: &Rc<ConfiguredModelDescriptor>) -> Rc<RefCell<Self>> {
        let model = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            collections: Rc::clone(collections),
            descriptor: Rc::clone(descriptor),
            parameters: HashMap::new(),
            auto_reference: Weak::new(),
        }));

        model.borrow_mut().auto_reference = Rc::downgrade(&model);

        model
    }

    pub fn collections(&self) -> &Rc<CollectionPool> {
        &self.collections
    }

    pub fn descriptor(&self) -> &Rc<ConfiguredModelDescriptor> {
        &self.descriptor
    }

    pub fn add_parameter(&mut self, name: &str) -> Result<Rc<RefCell<Parameter>>, LogicError> {
        
        if self.descriptor.core_model().parameters().contains_key(name) {
            let parameter = Parameter::new( &(Rc::clone(&self.descriptor) as Rc<dyn ParameterizedDescriptor>), 
                                            &(Rc::clone(&self.descriptor.core_model()) as Rc<dyn ParameterizedDescriptor>),
                                            name
                                        );
            let rc_parameter = Rc::new(RefCell::new(parameter));

            if self.parameters.insert(name.to_string(), Rc::clone(&rc_parameter)).is_none() {
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

    pub fn parameters(&self) -> &HashMap<String, Rc<RefCell<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> Result<(), LogicError> {

        for (_, param) in &self.parameters {
            param.borrow().validate()?;
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
        if let Some(_forbidden_context) = self.parameters.iter().find(|&(_param_name, param)| !matches!(param.borrow().value(), Some(Value::Context{..}))) {
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
