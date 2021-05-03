

use std::fmt::Debug;
use super::super::super::executive::environment::{Environment, ContextualEnvironment};

pub trait Builder : Debug {
    
    fn static_build(&self, environment: &dyn Environment);
    fn dynamic_build(&self,  environment: &dyn ContextualEnvironment);
}
