
use std::rc::Rc;
use std::cell::RefCell;
use super::Builder;
use super::super::designer::SequenceDesigner;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};

#[derive(Debug)]
pub struct SequenceBuilder {
    designer: Rc<RefCell<SequenceDesigner>>
}

impl SequenceBuilder {
    pub fn new(designer: &Rc<RefCell<SequenceDesigner>>) -> Self {
        Self {
            designer: Rc::clone(designer)
        }
    }
}

impl Builder for SequenceBuilder {

    fn static_build(&self, environment: &dyn GenesisEnvironment) {

    }

    fn dynamic_build(&self,  environment: &dyn ContextualEnvironment) {

    }
}


