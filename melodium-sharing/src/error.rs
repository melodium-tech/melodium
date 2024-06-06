use melodium_common::descriptor::{Identifier, Status};
use melodium_engine::LogicError;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct SharingError {
    pub id: u32,
    pub kind: SharingErrorKind,
}

#[derive(Debug, Clone)]
pub enum SharingErrorKind {
    CompiledModel { identifier: Identifier },
    CompiledTreatment { identifier: Identifier },
    NoModelDesignAvailable { identifier: Identifier },
    NoTreatmentDesignAvailable { identifier: Identifier },
    InvalidIdentifier { wrong_identifier: crate::Identifier },
    MissingBaseIdentifier { model_identifier: Identifier },
    DataSerializationFailure {},
    Logic { error: LogicError },
}

impl Display for SharingErrorKind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SharingErrorKind::CompiledModel { identifier } => write!(
                f,
                "Model '{identifier}' is compiled and cannot be restitued"
            ),
            SharingErrorKind::CompiledTreatment { identifier } => write!(
                f,
                "Treatment '{identifier}' is compiled and cannot be restitued"
            ),
            SharingErrorKind::NoModelDesignAvailable { identifier } => {
                write!(f, "No design available for model '{identifier}'")
            }
            SharingErrorKind::NoTreatmentDesignAvailable { identifier } => {
                write!(f, "No design available for treatment '{identifier}'")
            }
            SharingErrorKind::InvalidIdentifier { wrong_identifier } => {
                write!(f, "Identifier '{wrong_identifier}' is invalid")
            }
            SharingErrorKind::MissingBaseIdentifier { model_identifier } => {
                write!(f, "Base identifier is missing for '{model_identifier}'")
            }
            SharingErrorKind::DataSerializationFailure {} => {
                write!(f, "Serialized data not managed")
            }
            SharingErrorKind::Logic { error } => write!(f, "{}", error),
        }
    }
}

impl SharingError {
    pub fn compiled_model(id: u32, identifier: Identifier) -> Self {
        Self {
            id,
            kind: SharingErrorKind::CompiledModel { identifier },
        }
    }

    pub fn compiled_treatment(id: u32, identifier: Identifier) -> Self {
        Self {
            id,
            kind: SharingErrorKind::CompiledTreatment { identifier },
        }
    }

    pub fn no_model_design_available(id: u32, identifier: Identifier) -> Self {
        Self {
            id,
            kind: SharingErrorKind::NoModelDesignAvailable { identifier },
        }
    }

    pub fn no_treatment_design_available(id: u32, identifier: Identifier) -> Self {
        Self {
            id,
            kind: SharingErrorKind::NoTreatmentDesignAvailable { identifier },
        }
    }

    pub fn invalid_identifier(id: u32, wrong_identifier: crate::Identifier) -> Self {
        Self {
            id,
            kind: SharingErrorKind::InvalidIdentifier { wrong_identifier },
        }
    }

    pub fn missing_base_identifier(id: u32, model_identifier: Identifier) -> Self {
        Self {
            id,
            kind: SharingErrorKind::MissingBaseIdentifier { model_identifier },
        }
    }

    pub fn data_serialization_error(id: u32) -> Self {
        Self {
            id,
            kind: SharingErrorKind::DataSerializationFailure {},
        }
    }

    pub fn logic(id: u32, error: LogicError) -> Self {
        Self {
            id,
            kind: SharingErrorKind::Logic { error },
        }
    }
}

impl Display for SharingError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "R{:04}: {}", self.id, self.kind)
    }
}

impl From<LogicError> for SharingError {
    fn from(le: LogicError) -> Self {
        SharingError::logic(0, le)
    }
}

impl Error for SharingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub type SharingErrors = Vec<SharingError>;
pub type SharingResult<T> = Status<T, SharingError, SharingError>;

impl From<SharingError> for SharingErrors {
    fn from(value: SharingError) -> Self {
        vec![value]
    }
}
