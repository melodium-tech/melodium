use std::sync::Arc;
use melodium_common::{descriptor::Treatment, executive::TrackId};

#[derive(Debug, Clone)]
pub enum HostTreatment {
    None,
    Treatment(Arc<dyn Treatment>),
    Direct(TrackId),
}