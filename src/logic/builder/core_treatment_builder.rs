
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::Builder;
use super::super::descriptor::CoreTreatmentDescriptor;
use super::super::super::executive::environment::{Environment, ContextualEnvironment};

#[derive(Debug)]
pub struct CoreTreatmentBuilder {

}

impl Builder for CoreTreatmentBuilder {

    fn static_build(&self, environment: &dyn Environment) {

    }

    fn dynamic_build(&self,  environment: &dyn ContextualEnvironment) {

    }
}
