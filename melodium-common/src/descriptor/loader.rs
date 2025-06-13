use crate::descriptor::{
    Context, Function, Identifier, IdentifierRequirement, Model, PackageRequirement, Status,
    Treatment,
};
use core::fmt::{Debug, Display, Formatter};
use downcast_rs::{impl_downcast, Downcast};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone)]
pub enum LoadingErrorKind {
    NoPackage {
        package_requirement: PackageRequirement,
    },
    NoEntryPointProvided,
    UnreachableFile {
        path: PathBuf,
        error: String,
    },
    WrongConfiguration {
        package: String,
    },
    NotFound {
        element: String,
    },
    CircularReference {
        identifier: IdentifierRequirement,
    },
    RepositoryError {
        error: Arc<dyn RepositoryError>,
    },
    ContentError {
        error: Arc<dyn ContentError>,
    },
    ContextExpected {
        expecter: Option<Identifier>,
        identifier_requirement: IdentifierRequirement,
    },
    FunctionExpected {
        expecter: Option<Identifier>,
        identifier_requirement: IdentifierRequirement,
    },
    ModelExpected {
        expecter: Option<Identifier>,
        identifier_requirement: IdentifierRequirement,
    },
    TreatmentExpected {
        expecter: Option<Identifier>,
        identifier_requirement: IdentifierRequirement,
    },
    JeuFormatError {
        error: String,
    },
    MappingFormatError {
        error: String,
    },
    UncompatiblePlatform {
        platform: String,
    },
    LibraryLoadingError {
        path: PathBuf,
        error: String,
    },
}

impl Display for LoadingErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadingErrorKind::NoPackage {
                package_requirement,
            } => write!(f, "No package '{package_requirement}' found"),
            LoadingErrorKind::NoEntryPointProvided => write!(f, "No entry point provided"),
            LoadingErrorKind::UnreachableFile { path, error } => write!(
                f,
                "File '{path}' cannot be read: {error}",
                path = path.to_string_lossy()
            ),
            LoadingErrorKind::WrongConfiguration { package } => {
                write!(f, "Package '{package}' have wrong configuration")
            }
            LoadingErrorKind::NotFound { element } => write!(f, "Element '{element}' not found"),
            LoadingErrorKind::CircularReference { identifier } => {
                write!(f, "Element '{identifier}' cause a circular reference")
            }
            LoadingErrorKind::RepositoryError { error } => write!(f, "{error}"),
            LoadingErrorKind::ContentError { error } => write!(f, "{error}"),
            LoadingErrorKind::ContextExpected {
                expecter,
                identifier_requirement,
            } => match expecter {
                Some(expecter) => write!(
                    f,
                    "'{expecter}' expected a context, but '{identifier_requirement}' is not"
                ),
                None => write!(f, "'{identifier_requirement}' is not a context"),
            },
            LoadingErrorKind::FunctionExpected {
                expecter,
                identifier_requirement,
            } => match expecter {
                Some(expecter) => write!(
                    f,
                    "'{expecter}' expected a function, but '{identifier_requirement}' is not"
                ),
                None => write!(f, "'{identifier_requirement}' is not a function"),
            },
            LoadingErrorKind::ModelExpected {
                expecter,
                identifier_requirement,
            } => match expecter {
                Some(expecter) => write!(
                    f,
                    "'{expecter}' expected a model, but '{identifier_requirement}' is not"
                ),
                None => write!(f, "'{identifier_requirement}' is not a model"),
            },
            LoadingErrorKind::TreatmentExpected {
                expecter,
                identifier_requirement,
            } => match expecter {
                Some(expecter) => write!(
                    f,
                    "'{expecter}' expected a treatment, but '{identifier_requirement}' is not"
                ),
                None => write!(f, "'{identifier_requirement}' is not a treatment"),
            },
            LoadingErrorKind::JeuFormatError { error } => {
                write!(f, "Jeu data cannot be processed: {error}")
            }
            LoadingErrorKind::MappingFormatError { error } => {
                write!(f, "Mapped package cannot be processed: {error}")
            }
            LoadingErrorKind::UncompatiblePlatform { platform } => {
                write!(f, "Platform '{platform}' is not compatible")
            }
            LoadingErrorKind::LibraryLoadingError { path, error } => write!(
                f,
                "Loading '{path}' failed: {error}",
                path = path.to_string_lossy()
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadingError {
    pub id: u32,
    pub kind: LoadingErrorKind,
}

impl LoadingError {
    pub fn no_package(id: u32, package_requirement: PackageRequirement) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::NoPackage {
                package_requirement,
            },
        }
    }

    pub fn no_entry_point_provided(id: u32) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::NoEntryPointProvided,
        }
    }

    pub fn unreachable_file(id: u32, path: PathBuf, error: String) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::UnreachableFile { path, error },
        }
    }

    pub fn wrong_configuration(id: u32, package: String) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::WrongConfiguration { package },
        }
    }

    pub fn not_found(id: u32, element: String) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::NotFound { element },
        }
    }

    pub fn circular_reference(id: u32, identifier: IdentifierRequirement) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::CircularReference { identifier },
        }
    }

    pub fn repository_error(id: u32, error: Arc<dyn RepositoryError>) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::RepositoryError { error },
        }
    }

    pub fn content_error(id: u32, error: Arc<dyn ContentError>) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::ContentError { error },
        }
    }

    pub fn context_expected(
        id: u32,
        expecter: Option<Identifier>,
        identifier_requirement: IdentifierRequirement,
    ) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::ContextExpected {
                expecter,
                identifier_requirement,
            },
        }
    }

    pub fn function_expected(
        id: u32,
        expecter: Option<Identifier>,
        identifier_requirement: IdentifierRequirement,
    ) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::FunctionExpected {
                expecter,
                identifier_requirement,
            },
        }
    }

    pub fn model_expected(
        id: u32,
        expecter: Option<Identifier>,
        identifier_requirement: IdentifierRequirement,
    ) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::ModelExpected {
                expecter,
                identifier_requirement,
            },
        }
    }

    pub fn treatment_expected(
        id: u32,
        expecter: Option<Identifier>,
        identifier_requirement: IdentifierRequirement,
    ) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::TreatmentExpected {
                expecter,
                identifier_requirement,
            },
        }
    }

    pub fn jeu_format_error(id: u32, error: String) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::JeuFormatError { error },
        }
    }

    pub fn mapping_format_error(id: u32, error: String) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::MappingFormatError { error },
        }
    }

    pub fn uncompatible_platform(id: u32, platform: String) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::UncompatiblePlatform { platform },
        }
    }

    pub fn library_loading_error(id: u32, path: PathBuf, error: String) -> Self {
        Self {
            id,
            kind: LoadingErrorKind::LibraryLoadingError { path, error },
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
    fn load_context(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Arc<dyn Context>>;
    fn load_function(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Arc<dyn Function>>;
    fn load_model(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Arc<dyn Model>>;
    fn load_treatment(
        &self,
        identifier_requirement: &IdentifierRequirement,
    ) -> LoadingResult<Arc<dyn Treatment>>;
}

pub trait RepositoryError: Display + Debug + Downcast + Send + Sync {}
impl_downcast!(RepositoryError);

pub type RepositoryErrors = Vec<Arc<dyn RepositoryError>>;

pub trait ContentError: Display + Debug + Downcast + Send + Sync {}
impl_downcast!(ContentError);

pub type ContentErrors = Vec<Arc<dyn ContentError>>;
