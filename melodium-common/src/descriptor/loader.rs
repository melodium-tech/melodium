
use crate::descriptor::{Context, Function, Identifier, Model, Treatment};
use std::sync::Arc;

pub enum LoadingError {
    NoPackage,
    NotFound,
    CircularReference,
    ContentError,
    ContextExpected,
    FunctionExpected,
    ModelExpected,
    TreatmentExpected,
}

pub trait Loader {
    fn load_context(&self, identifier: &Identifier) -> Result<Arc<Context>, LoadingError>;
    fn load_function(&self, identifier: &Identifier) -> Result<Arc<dyn Function>, LoadingError>;
    fn load_model(&self, identifier: &Identifier) -> Result<Arc<dyn Model>, LoadingError>;
    fn load_treatment(&self, identifier: &Identifier) -> Result<Arc<dyn Treatment>, LoadingError>;
}
