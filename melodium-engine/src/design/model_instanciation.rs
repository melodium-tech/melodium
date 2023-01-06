
use core::fmt::{Debug};
use super::Parameter;
use melodium_common::descriptor::Model;
use std::sync::Weak;
use std::collections::{HashMap};

#[derive(Debug)]
pub struct ModelInstanciation {
    pub name: String,
    pub descriptor: Weak<dyn Model>,
    pub parameters: HashMap<String, Parameter>,
}
