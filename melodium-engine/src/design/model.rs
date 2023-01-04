
use core::fmt::{Debug};
use super::Parameter;
use crate::descriptor::Model as ModelDescriptor;
use std::sync::Weak;
use std::collections::{HashMap};


#[derive(Debug)]
pub struct Model {
    pub descriptor: Weak<ModelDescriptor>,
    pub parameters: HashMap<String, Parameter>,
}
