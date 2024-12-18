use super::Parameter;
use crate::descriptor::Model as ModelDescriptor;
use core::fmt::Debug;
use melodium_common::descriptor::{Identified, Identifier, Model as _, Parameterized};
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
            .map(|desc| {
                desc.identifier() == identifier
                    || desc.parameters().iter().any(|(_, param)| {
                        param
                            .described_type()
                            .final_type()
                            .data()
                            .map(|dt| dt.identifier() == identifier)
                            .unwrap_or(false)
                    })
            })
            .unwrap_or(false)
            || self
                .parameters
                .iter()
                .any(|(_, parameter)| parameter.make_use(identifier))
    }

    pub fn uses(&self) -> Vec<Identifier> {
        let descriptor = self.descriptor.upgrade().unwrap();
        let mut uses = vec![];
        if let Some(base) = descriptor.base_model() {
            uses.push(base.identifier().clone());
        }
        uses.extend(descriptor.parameters().iter().filter_map(|(_, param)| {
            param
                .described_type()
                .final_type()
                .data()
                .map(|data| data.identifier().clone())
        }));
        uses.extend(
            self.parameters
                .iter()
                .flat_map(|(_, parameter)| parameter.uses()),
        );
        uses
    }
}
