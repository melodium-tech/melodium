
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::Builder;
use super::super::descriptor::CoreModelDescriptor;
use super::super::super::executive::environment::{Environment, ContextualEnvironment};

#[derive(Debug)]
pub struct CoreModelBuilder {

}

impl Builder for CoreModelBuilder {

    fn static_build(&self, environment: &dyn Environment) {

    }

    fn dynamic_build(&self,  environment: &dyn ContextualEnvironment) {

    }
}
