use crate::descriptor::{Context, Function, Identifier, Model, Treatment};
use core::fmt::{Debug, Display, Formatter};
use downcast_rs::{impl_downcast, Downcast};
use std::sync::Arc;

use super::Status;

#[derive(Debug, Clone)]
pub enum LoadingErrorKind {
    NoPackage {
        name: String,
    },
    NotFound {
        element: String,
    },
    CircularReference {
        identifier: Identifier,
    },
    ContentError {
        error: Arc<dyn ContentError>,
    },
    ContextExpected {
        expecter: Option<Identifier>,
        identifier: Identifier,
    },
    FunctionExpected {
        expecter: Option<Identifier>,
        identifier: Identifier,
    },
    ModelExpected {
        expecter: Option<Identifier>,
        identifier: Identifier,
    },
    TreatmentExpected {
        expecter: Option<Identifier>,
        identifier: Identifier,
    },
}

impl Display for LoadingErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadingErrorKind::NoPackage { name } => write!(f, "No package '{name}' found"),
            LoadingErrorKind::NotFound { element } => write!(f, "Element '{element}' not found"),
            LoadingErrorKind::CircularReference { identifier } => {
                write!(f, "Element '{identifier}' cause a circular reference")
            }
            LoadingErrorKind::ContentError { error } => write!(f, "{error}"),
            LoadingErrorKind::ContextExpected {
                expecter,
                identifier,
            } => match expecter {
                Some(expecter) => write!(
                    f,
                    "'{expecter}' expected a context, but '{identifier}' is not"
                ),
                None => write!(f, "'{identifier}' is not a context"),
            },
            LoadingErrorKind::FunctionExpected {
                expecter,
                identifier,
            } => match expecter {
                Some(expecter) => write!(
                    f,
                    "'{expecter}' expected a function, but '{identifier}' is not"
                ),
                None => write!(f, "'{identifier}' is not a function"),
            },
            LoadingErrorKind::ModelExpected {
                expecter,
                identifier,
            } => match expecter {
                Some(expecter) => write!(
                    f,
                    "'{expecter}' expected a model, but '{identifier}' is not"
                ),
                None => write!(f, "'{identifier}' is not a model"),
            },
            LoadingErrorKind::TreatmentExpected {
                expecter,
                identifier,
            } => match expecter {
                Some(expecter) => write!(
                    f,
                    "'{expecter}' expected a treatment, but '{identifier}' is not"
                ),
                None => write!(f, "'{identifier}' is not a treatment"),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadingError {
    pub id: u32,
    pub kind: LoadingErrorKind,
}

impl LoadingError {
    pub fn no_package(id: u32, name: String) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::NoPackage { name },
        }
    }

    pub fn not_found(id: u32, element: String) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::NotFound { element },
        }
    }

    pub fn circular_reference(id: u32, identifier: Identifier) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::CircularReference { identifier },
        }
    }

    pub fn content_error(id: u32, error: Arc<dyn ContentError>) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::ContentError { error },
        }
    }

    pub fn context_expected(id: u32, expecter: Option<Identifier>, identifier: Identifier) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::ContextExpected {
                expecter,
                identifier,
            },
        }
    }

    pub fn function_expected(
        id: u32,
        expecter: Option<Identifier>,
        identifier: Identifier,
    ) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::FunctionExpected {
                expecter,
                identifier,
            },
        }
    }

    pub fn model_expected(id: u32, expecter: Option<Identifier>, identifier: Identifier) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::ModelExpected {
                expecter,
                identifier,
            },
        }
    }

    pub fn treatment_expected(
        id: u32,
        expecter: Option<Identifier>,
        identifier: Identifier,
    ) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::TreatmentExpected {
                expecter,
                identifier,
            },
        }
    }
}

impl Display for LoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "L{:04}: {}", self.id, self.kind)
    }
}

pub type LoadingErrors = Vec<LoadingError>;
pub type LoadingResult<T> = Status<T, LoadingError, LoadingError>;

pub trait Loader {
    fn load_context(&self, identifier: &Identifier) -> LoadingResult<Arc<dyn Context>>;
    fn load_function(&self, identifier: &Identifier) -> LoadingResult<Arc<dyn Function>>;
    fn load_model(&self, identifier: &Identifier) -> LoadingResult<Arc<dyn Model>>;
    fn load_treatment(&self, identifier: &Identifier) -> LoadingResult<Arc<dyn Treatment>>;
}

pub trait ContentError: Display + Debug + Downcast + Send + Sync {}
impl_downcast!(ContentError);

pub type ContentErrors = Vec<Box<dyn ContentError>>;
