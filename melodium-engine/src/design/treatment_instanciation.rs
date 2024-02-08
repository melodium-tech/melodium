use super::Parameter;
use core::fmt::Debug;
use melodium_common::descriptor::{Attribuable, Attributes, DescribedType, Identifier, Treatment};
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct TreatmentInstanciation {
    pub name: String,
    pub descriptor: Weak<dyn Treatment>,
    pub generics: HashMap<String, DescribedType>,
    pub models: HashMap<String, String>,
    pub parameters: HashMap<String, Parameter>,
    pub attributes: Attributes,
}

impl TreatmentInstanciation {
    pub fn make_use(&self, identifier: &Identifier) -> bool {
        self.descriptor
            .upgrade()
            .map(|desc| desc.identifier() == identifier)
            .unwrap_or(false)
            || self
                .parameters
                .iter()
                .any(|(_, parameter)| parameter.make_use(identifier))
            || self.generics.iter().any(|(_, dt)| {
                dt.final_type()
                    .data()
                    .map(|data| data.identifier() == identifier)
                    .unwrap_or(false)
            })
    }

    pub fn uses(&self) -> Vec<Identifier> {
        let mut uses = vec![self.descriptor.upgrade().unwrap().identifier().clone()];
        uses.extend(
            self.generics
                .iter()
                .filter_map(|(_, dt)| dt.final_type().data().map(|data| data.identifier().clone())),
        );
        uses.extend(
            self.parameters
                .iter()
                .flat_map(|(_, parameter)| parameter.uses()),
        );
        uses
    }
}

impl Attribuable for TreatmentInstanciation {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}
