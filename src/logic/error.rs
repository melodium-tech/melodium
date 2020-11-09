
pub enum LogicErrorKind {
    UnexistingVariable,
    UnexistingContext,
    UnexistingContextVariable,
    UnmatchingDataType,
}

pub struct LogicError {
    pub kind: LogicErrorKind
}

impl LogicError {

    pub fn unexisting_variable() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingVariable
        }
    }

    pub fn unexisting_context() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingContext
        }
    }

    pub fn unexisting_context_variable() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingContextVariable
        }
    }

    pub fn unmatching_datatype() -> Self {
        Self {
            kind: LogicErrorKind::UnmatchingDataType
        }
    }
}
