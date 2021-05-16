

use std::collections::HashMap;
use std::fmt::Debug;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::value::Value;

pub trait Builder : Debug {
    
    fn static_build(&self, environment: &dyn GenesisEnvironment);
    fn dynamic_build(&self,  environment: &dyn ContextualEnvironment);
}
