use super::Parameter;
use core::fmt::Debug;
use melodium_common::descriptor::{Identifier, Model, Attribuable, Attributes};
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct ModelInstanciation {
    pub name: String,
    pub descriptor: Weak<dyn Model>,
    pub parameters: HashMap<String, Parameter>,
    pub attributes: Attributes,
}

impl ModelInstanciation {
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

impl Attribuable for ModelInstanciation {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}
