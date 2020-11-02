
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use super::super::error::LogicError;
use super::super::collection_pool::CollectionPool;
use super::super::ConfiguredModelDescriptor;
use super::parameter::Parameter;

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

    pub fn validate(&self) -> Result<(), LogicError> {
        Ok(())
    }

    pub fn register(&self) -> Result<(), LogicError> {
        
        self.validate()?;

        todo!();
    }
}
