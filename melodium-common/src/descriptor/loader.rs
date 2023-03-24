use crate::descriptor::{Context, Function, Identifier, Model, Treatment};
use std::sync::Arc;

#[derive(Debug)]
pub enum LoadingError {
    NoPackage,
    NotFound(u32),
    CircularReference,
    ContentError,
    ContextExpected,
    FunctionExpected,
    ModelExpected,
    TreatmentExpected,
}

pub trait Loader {
    fn load_context(&self, identifier: &Identifier) -> Result<Arc<dyn Context>, LoadingError>;
    fn load_function(&self, identifier: &Identifier) -> Result<Arc<dyn Function>, LoadingError>;
    fn load_model(&self, identifier: &Identifier) -> Result<Arc<dyn Model>, LoadingError>;
    fn load_treatment(&self, identifier: &Identifier) -> Result<Arc<dyn Treatment>, LoadingError>;
}
