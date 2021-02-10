
pub enum LogicErrorKind {
    UnexistingVariable,
    UnexistingContext,
    UnexistingContextVariable,
    UnexistingParameter,
    UnmatchingDataType,
    UnsetParameter,
    MultipleParameterAssignation,
    NoValue,
    NoContext,
    UnavailableContext,
    ConnectionInputRequired,
    ConnectionInputForbidden,
    ConnectionInputNotFound,
    ConnectionInputUnmatchingDataType,
    ConnectionInputNotSet,
    ConnectionSelfInputNotFound,
    ConnectionOutputRequired,
    ConnectionOutputForbidden,
    ConnectionOutputNotFound,
    ConnectionOutputUnmatchingDataType,
    ConnectionOutputNotSet,
    ConnectionSelfOutputNotFound,
    UnexistingTreatment,
    UnexistingModel,
    UndeclaredTreatment,
    UnexistingConnectionType,
    UnsatisfiedOutput,
    OverloadedOutput,
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

    pub fn unexisting_parameter() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingParameter
        }
    }

    pub fn unmatching_datatype() -> Self {
        Self {
            kind: LogicErrorKind::UnmatchingDataType
        }
    }

    pub fn unset_parameter() -> Self {
        Self {
            kind: LogicErrorKind::UnsetParameter
        }
    }

    pub fn multiple_parameter_assignation() -> Self {
        Self {
            kind: LogicErrorKind::MultipleParameterAssignation
        }
    }

    pub fn no_value() -> Self {
        Self {
            kind: LogicErrorKind::NoValue
        }
    }

    pub fn no_context() -> Self {
        Self {
            kind: LogicErrorKind::NoContext
        }
    }

    pub fn unavailable_context() -> Self {
        Self {
            kind: LogicErrorKind::UnavailableContext
        }
    }

    pub fn connection_input_required() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputRequired
        }
    }

    pub fn connection_input_forbidden() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputForbidden
        }
    }

    pub fn connection_input_not_found() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputNotFound
        }
    }

    pub fn connection_input_unmatching_datatype() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputUnmatchingDataType
        }
    }

    pub fn connection_input_not_set() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputNotSet
        }
    }

    pub fn connection_self_input_not_found() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionSelfInputNotFound
        }
    }

    pub fn connection_output_required() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputRequired
        }
    }

    pub fn connection_output_forbidden() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputForbidden
        }
    }

    pub fn connection_output_not_found() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputNotFound
        }
    }

    pub fn connection_output_unmatching_datatype() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputUnmatchingDataType
        }
    }

    pub fn connection_output_not_set() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputNotSet
        }
    }

    pub fn connection_self_output_not_found() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionSelfOutputNotFound
        }
    }

    pub fn unexisting_treatment() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingTreatment
        }
    }

    pub fn unexisting_model() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingModel
        }
    }

    pub fn undeclared_treatment() -> Self {
        Self {
            kind: LogicErrorKind::UndeclaredTreatment
        }
    }

    pub fn unexisting_connexion_type() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingConnectionType
        }
    }

    pub fn unsatisfied_output() -> Self {
        Self {
            kind: LogicErrorKind::UnsatisfiedOutput
        }
    }

    pub fn overloaded_output() -> Self {
        Self {
            kind: LogicErrorKind::OverloadedOutput
        }
    }
}
