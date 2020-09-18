
use super::collection::Collection;
use super::descriptor::ModelConfigDescriptor;
use super::descriptor::ModelTypeDescriptor;
use super::descriptor::TreatmentDescriptor;

pub struct CollectionPool {
    pub models: Collection<ModelConfigDescriptor>,
    pub model_types: Collection<ModelTypeDescriptor>,
    pub treatments: Collection<dyn TreatmentDescriptor>,
}

impl CollectionPool {
    pub fn new() -> Self {
        Self {
            models: Collection::new(),
            model_types: Collection::new(),
            treatments: Collection::new(),
        }
    }
}
