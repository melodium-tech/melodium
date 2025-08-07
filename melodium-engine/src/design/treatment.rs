use super::{Connection, ModelInstanciation, TreatmentInstanciation};
use crate::descriptor::Treatment as TreatmentDescriptor;
use core::fmt::Debug;
use melodium_common::descriptor::Identifier;
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
        self.model_instanciations
            .iter()
            .any(|(_, model)| model.make_use(identifier))
            || self
                .treatments
                .iter()
                .any(|(_, treatment)| treatment.make_use(identifier))
    }

    pub fn uses(&self) -> Vec<Identifier> {
        let mut uses = vec![];
        uses.extend(
            self.model_instanciations
                .iter()
                .flat_map(|(_, mi)| mi.uses()),
        );
        uses.extend(self.treatments.iter().flat_map(|(_, ti)| ti.uses()));
        uses
    }
}
