use super::Parameter;
use core::fmt::Debug;
use melodium_common::descriptor::{Identifier, Treatment, Attribuable, Attributes};
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct TreatmentInstanciation {
    pub name: String,
    pub descriptor: Weak<dyn Treatment>,
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
    }
}

impl Attribuable for TreatmentInstanciation {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}
