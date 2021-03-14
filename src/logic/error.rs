
//! Provides Mélodium logic error management.
//! 
//! The main type of this module is [`LogicError`], which handles most of the management, combined with kind of errors detailed with [`LogicErrorKind`].

/// Kind of logic error that might happen.
pub enum LogicErrorKind {
    /// The referenced variable for value doesn't exist.
    UnexistingVariable,
    /// The referenced context to look for getting value doesn't exist.
    UnexistingContext,
    /// The referenced context variable for value doesn't exist.
    UnexistingContextVariable,
    /// The designated parameter doesn't exist in descriptor.
    UnexistingParameter,
    /// The value datatype doesn't match the required one.
    UnmatchingDataType,
    /// A parameter hasn't been set up compared to descriptor.
    UnsetParameter,
    /// A parameter is assigned multiple times.
    MultipleParameterAssignation,
    /// A parameter didn't get any value.
    NoValue,
    /// No context reference is allowed there.
    NoContext,
    /// The context referenced is not available in this scope.
    UnavailableContext,
    /// A connection input data is required, none provided.
    ConnectionInputRequired,
    /// A connection input data is provided, but none is allowed.
    ConnectionInputForbidden,
    /// The connection input data is not provided by input treatment.
    ConnectionInputNotFound,
    /// The connection input data type provided by input treatment doesn't match the connection descriptor.
    ConnectionInputUnmatchingDataType,
    /// The connection input treatment is not setted up.
    ConnectionInputNotSet,
    /// The connection input data is not provided in self inputs.
    ConnectionSelfInputNotFound,
    /// A connection output data is required, none provided.
    ConnectionOutputRequired,
    /// A connection output data is provided, but none is allowed.
    ConnectionOutputForbidden,
    /// The connection output data is not provided by output treatment.
    ConnectionOutputNotFound,
    /// The connection output data type provided by output treatment doesn't match the connection descriptor.
    ConnectionOutputUnmatchingDataType,
    /// The connection output treatment is not setted up.
    ConnectionOutputNotSet,
    /// The connection output data is not provided in self outputs.
    ConnectionSelfOutputNotFound,
    /// The treatment is not existing within current available treatments.
    UnexistingTreatment,
    /// The model is not existing within current available models.
    UnexistingModel,
    /// The treatment is not declared there.
    UndeclaredTreatment,
    /// The connection type is not existing within current available connections.
    UnexistingConnectionType,
    /// The sequence output is not currently satisfied, not connected to any treatment output.
    UnsatisfiedOutput,
    /// The sequence output is overloaded, having multiple treatment outputs connected to.
    OverloadedOutput,
}

/// Handles and describe a Mélodium logic error.
pub struct LogicError {
    /// Kind of error.
    pub kind: LogicErrorKind
}

impl LogicError {

    /// Generates a new error with [`LogicErrorKind::UnexistingVariable`] kind.
    pub fn unexisting_variable() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingVariable
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingContext`] kind.
    pub fn unexisting_context() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingContext
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingContextVariable`] kind.
    pub fn unexisting_context_variable() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingContextVariable
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingParameter`] kind.
    pub fn unexisting_parameter() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingParameter
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnmatchingDataType`] kind.
    pub fn unmatching_datatype() -> Self {
        Self {
            kind: LogicErrorKind::UnmatchingDataType
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnsetParameter`] kind.
    pub fn unset_parameter() -> Self {
        Self {
            kind: LogicErrorKind::UnsetParameter
        }
    }

    /// Generates a new error with [`LogicErrorKind::MultipleParameterAssignation`] kind.
    pub fn multiple_parameter_assignation() -> Self {
        Self {
            kind: LogicErrorKind::MultipleParameterAssignation
        }
    }

    /// Generates a new error with [`LogicErrorKind::NoValue`] kind.
    pub fn no_value() -> Self {
        Self {
            kind: LogicErrorKind::NoValue
        }
    }

    /// Generates a new error with [`LogicErrorKind::NoContext`] kind.
    pub fn no_context() -> Self {
        Self {
            kind: LogicErrorKind::NoContext
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnavailableContext`] kind.
    pub fn unavailable_context() -> Self {
        Self {
            kind: LogicErrorKind::UnavailableContext
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionInputRequired`] kind.
    pub fn connection_input_required() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputRequired
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionInputForbidden`] kind.
    pub fn connection_input_forbidden() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputForbidden
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionInputNotFound`] kind.
    pub fn connection_input_not_found() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputNotFound
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionInputUnmatchingDataType`] kind.
    pub fn connection_input_unmatching_datatype() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputUnmatchingDataType
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionInputNotSet`] kind.
    pub fn connection_input_not_set() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionInputNotSet
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionSelfInputNotFound`] kind.
    pub fn connection_self_input_not_found() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionSelfInputNotFound
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionOutputRequired`] kind.
    pub fn connection_output_required() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputRequired
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionOutputForbidden`] kind.
    pub fn connection_output_forbidden() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputForbidden
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionOutputNotFound`] kind.
    pub fn connection_output_not_found() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputNotFound
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionOutputUnmatchingDataType`] kind.
    pub fn connection_output_unmatching_datatype() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputUnmatchingDataType
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionOutputNotSet`] kind.
    pub fn connection_output_not_set() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionOutputNotSet
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionSelfOutputNotFound`] kind.
    pub fn connection_self_output_not_found() -> Self {
        Self {
            kind: LogicErrorKind::ConnectionSelfOutputNotFound
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingTreatment`] kind.
    pub fn unexisting_treatment() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingTreatment
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingModel`] kind.
    pub fn unexisting_model() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingModel
        }
    }

    /// Generates a new error with [`LogicErrorKind::UndeclaredTreatment`] kind.
    pub fn undeclared_treatment() -> Self {
        Self {
            kind: LogicErrorKind::UndeclaredTreatment
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingConnectionType`] kind.
    pub fn unexisting_connexion_type() -> Self {
        Self {
            kind: LogicErrorKind::UnexistingConnectionType
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnsatisfiedOutput`] kind.
    pub fn unsatisfied_output() -> Self {
        Self {
            kind: LogicErrorKind::UnsatisfiedOutput
        }
    }

    /// Generates a new error with [`LogicErrorKind::OverloadedOutput`] kind.
    pub fn overloaded_output() -> Self {
        Self {
            kind: LogicErrorKind::OverloadedOutput
        }
    }
}
