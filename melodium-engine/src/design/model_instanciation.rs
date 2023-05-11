use super::Parameter;
use core::fmt::Debug;
use melodium_common::descriptor::Model;
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct ModelInstanciation {
    pub name: String,
    pub descriptor: Weak<dyn Model>,
    pub parameters: HashMap<String, Parameter>,
}
