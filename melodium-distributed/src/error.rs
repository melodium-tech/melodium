use melodium_common::descriptor::{LoadingError, Status};
use melodium_sharing::SharingError;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct DistributionError {
    pub id: u32,
    pub kind: DistributionErrorKind,
}

#[derive(Debug, Clone)]
pub enum DistributionErrorKind {
    Loading { error: LoadingError },
    Sharing { error: SharingError },
}

impl Display for DistributionErrorKind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            DistributionErrorKind::Loading { error } => write!(f, "{}", error),
            DistributionErrorKind::Sharing { error } => write!(f, "{}", error),
        }
    }
}

impl DistributionError {
    pub fn loading(id: u32, error: LoadingError) -> Self {
        Self {
            id,
            kind: DistributionErrorKind::Loading { error },
        }
    }

    pub fn sharing(id: u32, error: SharingError) -> Self {
        Self {
            id,
            kind: DistributionErrorKind::Sharing { error },
        }
    }
}

impl Display for DistributionError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "T{:04}: {}", self.id, self.kind)
    }
}

impl From<LoadingError> for DistributionError {
    fn from(le: LoadingError) -> Self {
        DistributionError::loading(0, le)
    }
}

impl From<SharingError> for DistributionError {
    fn from(se: SharingError) -> Self {
        DistributionError::sharing(0, se)
    }
}

impl Error for DistributionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub type DistributionErrors = Vec<DistributionError>;
pub type DistributionResult<T> = Status<T, DistributionError, DistributionError>;

impl From<DistributionError> for DistributionErrors {
    fn from(value: DistributionError) -> Self {
        vec![value]
    }
}
