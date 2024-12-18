use melodium_common::descriptor::{Identifier, Treatment};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum HostTreatment {
    Treatment(Arc<dyn Treatment>),
    Direct,
}

impl HostTreatment {
    pub fn host_id(&self) -> Option<&Identifier> {
        match self {
            HostTreatment::Treatment(descriptor) => Some(descriptor.identifier()),
            _ => None,
        }
    }
}
