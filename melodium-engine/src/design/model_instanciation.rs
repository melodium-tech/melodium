use super::Parameter;
use core::fmt::Debug;
use melodium_common::descriptor::{Attribuable, Attributes, Identifier, Model};
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

    pub fn uses(&self) -> Vec<Identifier> {
        let descriptor = self.descriptor.upgrade().unwrap();
        let mut uses = vec![descriptor.identifier().clone()];
        uses.extend(
            self.parameters
                .iter()
                .flat_map(|(_, parameter)| parameter.uses()),
        );
        uses
    }
}

impl Attribuable for ModelInstanciation {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}
