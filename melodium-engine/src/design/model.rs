use super::Parameter;
use crate::descriptor::Model as ModelDescriptor;
use core::fmt::Debug;
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct Model {
    pub descriptor: Weak<ModelDescriptor>,
    pub parameters: HashMap<String, Parameter>,
}
