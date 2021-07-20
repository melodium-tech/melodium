

use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::Debug;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::value::Value;
use super::super::super::executive::model::Model;
use super::super::super::executive::transmitter::Transmitter;

pub trait Builder : Debug {
    
    fn static_build(&self, environment: &dyn GenesisEnvironment) -> Option<Arc<dyn Model>>;
    fn dynamic_build(&self, environment: &dyn ContextualEnvironment) -> Option<HashMap<String, Transmitter>>;
}
