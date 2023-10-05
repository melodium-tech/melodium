use super::{Connection, ModelInstanciation, TreatmentInstanciation};
use crate::descriptor::Treatment as TreatmentDescriptor;
use core::fmt::Debug;
use melodium_common::descriptor::{Identified, Identifier};
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct Treatment {
    pub descriptor: Weak<TreatmentDescriptor>,
    pub model_instanciations: HashMap<String, ModelInstanciation>,
    pub treatments: HashMap<String, TreatmentInstanciation>,
    pub connections: Vec<Connection>,
}

impl Treatment {
    pub fn make_use(&self, identifier: &Identifier) -> bool {
        self.descriptor
            .upgrade()
            .map(|desc| desc.identifier() == identifier)
            .unwrap_or(false)
            || self
                .model_instanciations
                .iter()
                .any(|(_, model)| model.make_use(identifier))
            || self
                .treatments
                .iter()
                .any(|(_, treatment)| treatment.make_use(identifier))
    }
}
