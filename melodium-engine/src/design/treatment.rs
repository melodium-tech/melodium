use super::{Connection, ModelInstanciation, TreatmentInstanciation};
use crate::descriptor::Treatment as TreatmentDescriptor;
use core::fmt::Debug;
use melodium_common::descriptor::{Identified, Identifier, Parameterized, Treatment as _};
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
            .map(|desc| {
                desc.identifier() == identifier
                    || desc
                        .contexts()
                        .iter()
                        .any(|(_, context)| context.identifier() == identifier)
                    || desc
                        .models()
                        .iter()
                        .any(|(_, model)| model.identifier() == identifier)
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
                .model_instanciations
                .iter()
                .any(|(_, model)| model.make_use(identifier))
            || self
                .treatments
                .iter()
                .any(|(_, treatment)| treatment.make_use(identifier))
    }

    pub fn uses(&self) -> Vec<Identifier> {
        let descriptor = self.descriptor.upgrade().unwrap();
        let mut uses = vec![];
        uses.extend(
            descriptor
                .contexts()
                .iter()
                .map(|(_, context)| context.identifier().clone()),
        );
        uses.extend(
            descriptor
                .models()
                .iter()
                .map(|(_, model)| model.identifier().clone()),
        );
        uses.extend(descriptor.parameters().iter().filter_map(|(_, param)| {
            param
                .described_type()
                .final_type()
                .data()
                .map(|data| data.identifier().clone())
        }));
        uses.extend(
            self.model_instanciations
                .iter()
                .flat_map(|(_, mi)| mi.uses()),
        );
        uses.extend(self.treatments.iter().flat_map(|(_, ti)| ti.uses()));
        uses
    }
}
