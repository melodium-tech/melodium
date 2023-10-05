use super::Parameter;
use crate::descriptor::Model as ModelDescriptor;
use core::fmt::Debug;
use melodium_common::descriptor::{Identified, Identifier};
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct Model {
    pub descriptor: Weak<ModelDescriptor>,
    pub parameters: HashMap<String, Parameter>,
}

impl Model {
    pub fn make_use(&self, identifier: &Identifier) -> bool {
        self.descriptor
            .upgrade()
            .map(|desc| desc.identifier() == identifier)
            .unwrap_or(false)
            || self
                .parameters
                .iter()
                .any(|(_, parameter)| parameter.make_use(identifier))
    }
}
