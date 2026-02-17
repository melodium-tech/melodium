use crate::{
    building::{BuildId, HostTreatment},
    world::World,
};
use melodium_common::descriptor::Treatment;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum TransmissionDebug {
    None,
    Basic(Arc<World>, TransmissionDetails),
    Detailed(Arc<World>, TransmissionDetails),
}

#[derive(Debug, Clone)]
pub struct TransmissionDetails {
    pub treatment: Arc<dyn Treatment>,
    pub host_treatment: HostTreatment,
    pub host_build: Option<BuildId>,
    pub build_id: BuildId,
    pub label: String,
    pub name: String,
}
