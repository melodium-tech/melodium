
use std::rc::Rc;
use std::cell::RefCell;
use super::Builder;
use super::super::designer::ModelDesigner;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};

#[derive(Debug)]
pub struct ConfiguredModelBuilder {
    designer: Rc<RefCell<ModelDesigner>>
}

impl ConfiguredModelBuilder {
    pub fn new(designer: &Rc<RefCell<ModelDesigner>>) -> Self {
        Self {
            designer: Rc::clone(designer)
        }
    }
}

impl Builder for ConfiguredModelBuilder {

    fn static_build(&self, environment: &dyn GenesisEnvironment) {

    }

    fn dynamic_build(&self,  environment: &dyn ContextualEnvironment) {

    }
}
