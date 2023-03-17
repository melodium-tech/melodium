
//! Provides common collections pool for logical environment.

use super::collection::Collection;
use super::descriptor::FunctionDescriptor;
use super::descriptor::ModelDescriptor;
use super::descriptor::TreatmentDescriptor;

#[derive(Debug, Clone)]
pub struct CollectionPool {
    pub functions: Collection<dyn FunctionDescriptor>,
    pub models: Collection<dyn ModelDescriptor>,
    pub treatments: Collection<dyn TreatmentDescriptor>,
}

impl CollectionPool {
    pub fn new() -> Self {
        Self {
            functions: Collection::new(),
            models: Collection::new(),
            treatments: Collection::new(),
        }
    }
}
