//! Provides Mélodium logic error management.
//!
//! The main type of this module is [`LogicError`], which handles most of the management, combined with kind of errors detailed with [`LogicErrorKind`].

use core::fmt::{Debug, Display};
use std::string::ToString;
use std::sync::Arc;

use melodium_common::descriptor::{DataTrait, DescribedType, Flow, Identifier, Status};

use crate::{building::CheckStep, design::Value, designer::Reference};

/// Kind of logic error that might happen.
#[derive(Debug, Clone)]
pub enum LogicErrorKind {
    /// Designer do not have collection defined.
    CollectionUndefined,
    /// Descriptor has not been commited, no designer available yet.
    UncommitedDescriptor { identifier: Identifier },
    /// No designer to commit from.
    NoDesigner { identifier: Identifier },
    /// Design have some errors, while pure success is expected.
    ErroneousDesign { identifier: Identifier },
    /// Checks hasn't all been sucessful, while pure sucess is expected.
    ErroneousChecks,
    /// No design available.
    UnavailableDesign { identifier: Identifier },
    /// The launch must be done using a treatment.
    LaunchExpectTreatment { wrong_identifier: Identifier },
    /// A parameter with wrong or missing value was given for launch.
    LaunchWrongParameter { parameter: String },
    /// The referenced variable for value doesn't exist.
    UnexistingVariable {
        identifier: Identifier,
        parameter: String,
        variable: String,
    },
    /// The referenced context variable for value doesn't exist.
    UnexistingContextVariable {
        identifier: Identifier,
        parameter: String,
        context: Identifier,
        variable: String,
    },
    /// The designated parameter doesn't exist in descriptor.
    UnexistingParameter {
        scope: Identifier,
        called: Identifier,
        parameter: String,
    },
    /// The value datatype doesn't match the required one.
    UnmatchingDataType {
        scope: Identifier,
        called: Identifier,
        parameter: String,
        value: Value,
        expected: DescribedType,
        given: DescribedType,
    },
    /// A parameter hasn't been set up compared to descriptor.
    UnsetParameter {
        scope: Identifier,
        called: Identifier,
        parameter: String,
    },
    /// A parameter is assigned multiple times.
    MultipleParameterAssignation {
        scope: Identifier,
        called: Identifier,
        parameter: String,
    },
    /// A parameter didn't get any value.
    NoValue {
        scope: Identifier,
        called: Identifier,
        parameter: String,
    },
    /// No context reference is allowed there.
    NoContext {
        scope: Identifier,
        model: Identifier,
        name: String,
        parameter: String,
    },
    /// The context referenced is not available in this scope.
    UnavailableContext {
        scope: Identifier,
        context: Identifier,
    },
    /// The connection input data is not provided by input treatment.
    ConnectionInputNotFound {
        scope: Identifier,
        to: Identifier,
        input: String,
    },
    /// The connection input data is not provided in self inputs.
    ConnectionSelfInputNotFound { scope: Identifier, input: String },
    /// The connection output data is not provided by output treatment.
    ConnectionOutputNotFound {
        scope: Identifier,
        from: Identifier,
        output: String,
    },
    /// The connection output data is not provided in self outputs.
    ConnectionSelfOutputNotFound { scope: Identifier, output: String },
    /// The treatment is not existing within current available treatments.
    UnexistingTreatment {
        scope: Identifier,
        claimed: Identifier,
    },
    /// The model is not existing within current available models.
    UnexistingModel {
        scope: Identifier,
        claimed: Identifier,
    },
    /// The context is not existing within current available contextes.
    UnexistingContext {
        scope: Identifier,
        claimed: Identifier,
    },
    /// The function is not existing within current available functions.
    UnexistingFunction {
        scope: Identifier,
        claimed: Identifier,
    },
    /// The model is not declared here.
    UndeclaredModel { scope: Identifier, model: String },
    /// The model name is already declared.
    AlreadyDeclaredModel { scope: Identifier, model: String },
    /// The treatment is not declared here.
    UndeclaredTreatment {
        scope: Identifier,
        treatment: String,
    },
    /// The treatment name is already declared.
    AlreadyDeclaredTreatment {
        scope: Identifier,
        treatment: String,
    },
    /// The connection type is not existing within current available connections.
    UnexistingConnectionType {
        scope: Identifier,
        from: String,
        output: String,
        to: String,
        input: String,
        output_flow: Flow,
        output_type: DescribedType,
        input_flow: Flow,
        input_type: DescribedType,
    },
    /// The sequence output is not currently satisfied, not connected to any treatment output.
    UnsatisfiedOutput { scope: Identifier, output: String },
    /// The sequence output is overloaded, having multiple treatment outputs connected to.
    OverloadedOutput { scope: Identifier, output: String },
    /// The (core) model type does not match.
    UnmatchingModelType {
        scope: Identifier,
        called: Identifier,
        name: String,
        expected: Identifier,
        given_name: String,
        given: Identifier,
    },
    /// There are no matching pararmetric model.
    UnexistingParametricModel {
        scope: Identifier,
        called: Identifier,
        parametric_model: String,
    },
    /// A model hasn't been set up compared to descriptor.
    UnsetModel {
        scope: Identifier,
        called: Identifier,
        parametric_model: String,
    },
    /// The build step is already included in the call stack, meaning there is an infinite call loop.
    AlreadyIncludedBuildStep {
        treatment: Identifier,
        cause_step: CheckStep,
        check_steps: Vec<CheckStep>,
    },
    /// The treatment input in not satisfied
    UnsatisfiedInput {
        scope: Option<Identifier>,
        treatment: String,
        input: String,
    },
    /// A constant is required but the value assigned is variable
    ConstRequiredVarProvided {
        scope: Identifier,
        called: Identifier,
        parameter: String,
        variable: String,
    },
    /// A constant is required but a context is provided
    ConstRequiredContextProvided {
        scope: Identifier,
        called: Identifier,
        parameter: String,
        context: Identifier,
        entry: String,
    },
    /// A model instanciation can only have const assignations
    ModelInstanciationConstOnly {
        scope: Identifier,
        called: Identifier,
        name: String,
        parameter: String,
    },
    /// A function returns var when const is expected, due to a var parameter
    ConstRequiredFunctionReturnsVar {
        scope: Identifier,
        called: Identifier,
        parameter: String,
        function: Identifier,
    },
    /// A function doesn't get the right number of parameters
    UnmatchingNumberOfParameters {
        scope: Identifier,
        function: Identifier,
    },
    /// A value is setup for a generic that doesn't exists
    UnexistingGeneric {
        scope: Identifier,
        element: Identifier,
        name: String,
        described_type: DescribedType,
    },
    /// A generic value is not defined
    UndefinedGeneric {
        scope: Identifier,
        element: Identifier,
        described_type: DescribedType,
    },
    /// Traits are not satisfied
    UnsatisfiedTraits {
        scope: Identifier,
        element: Identifier,
        described_type: DescribedType,
        unsatisfied_traits: Vec<DataTrait>,
    },
}

impl Display for LogicErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicErrorKind::CollectionUndefined => write!(f, "No collection defined"),
            LogicErrorKind::UncommitedDescriptor { identifier } => write!(f, "Uncommited descriptor, no designer available for '{identifier}'"),
            LogicErrorKind::NoDesigner {identifier } => write!(f, "Nothing to commit, as no designer available for '{identifier}'"),
            LogicErrorKind::ErroneousDesign {identifier} => write!(f, "Design for '{identifier}' contains errors and cannot be commited"),
            LogicErrorKind::ErroneousChecks => write!(f, "Building coherency checks found errors, build cannot be made"),
            LogicErrorKind::UnavailableDesign { identifier } => write!(f, "Unavailable design for '{identifier}'"),
            LogicErrorKind::LaunchExpectTreatment { wrong_identifier } => write!(f, "Launch must be done using a treatment, '{wrong_identifier}' is not one"),
            LogicErrorKind::LaunchWrongParameter { parameter } => write!(f, "Parameter '{parameter}' has no valid value for launch"),
            LogicErrorKind::UnexistingVariable {identifier,
                parameter,
                variable,} => write!(f, "Referenced '{variable}' variable for '{parameter}' parameter doesn't exist in '{identifier}'"),
            LogicErrorKind::UnexistingContextVariable{identifier,
                parameter,
                context,
                variable,} => write!(f, "Referenced '{variable}' context value for '{parameter}' does not exist within '{context}' context in '{identifier}'"),
            LogicErrorKind::UnexistingParameter {
                scope,
                called,
                parameter,
            } => write!(f, "Parameter '{parameter}' does not exist for '{called}' in '{scope}'"),
            LogicErrorKind::UnmatchingDataType {
                scope, called, parameter, value, expected, given
            } => write!(f, "Datatype does not match for '{parameter}' of '{called}' in '{scope}', {expected} expected but '{value}' is {given}"),
            LogicErrorKind::UnsetParameter { scope, called, parameter } => write!(f, "Parameter '{parameter}' of '{called}' is not set in '{scope}'"),
            LogicErrorKind::MultipleParameterAssignation { scope, called, parameter } => write!(f, "Parameter '{parameter}' of '{called}' assigned multiple times in '{scope}'"),
            LogicErrorKind::NoValue {scope, called, parameter} => write!(f, "No value assigned to '{parameter}' of '{called}' in '{scope}'"),
            LogicErrorKind::NoContext{scope, model, name, parameter } => write!(f, "Context used for parameter '{parameter}' of model '{name}' type '{model}' in '{scope}', but context values cannot be used for models"),
            LogicErrorKind::UnavailableContext{scope, context} => write!(f, "Context '{context}' not available in '{scope}'"),
            LogicErrorKind::ConnectionInputNotFound{scope: _, to, input} => write!(f, "Input '{input}' is not provided by '{to}'"),
            LogicErrorKind::ConnectionSelfInputNotFound { scope, input } => write!(f, "Input '{input}' does not exist for '{scope}'"),
            LogicErrorKind::ConnectionOutputNotFound { scope: _, from, output } => write!(f, "Output '{output}' is not provided by '{from}'"),
            LogicErrorKind::ConnectionSelfOutputNotFound{scope, output} => write!(f, "Output '{output}' does not exist for '{scope}'"),
            LogicErrorKind::UnexistingTreatment{scope: _, claimed} => write!(f, "Treatment '{claimed}' does not exist"),
            LogicErrorKind::UnexistingModel { scope: _, claimed } => write!(f, "Model '{claimed}' does not exist"),
            LogicErrorKind::UnexistingContext { scope: _, claimed } => write!(f, "Context '{claimed}' does not exist"),
            LogicErrorKind::UnexistingFunction{ scope: _, claimed } => write!(f, "Function '{claimed}' does not exist"),
            LogicErrorKind::UndeclaredModel { scope, model } => write!(f, "Model '{model}' is not declared in '{scope}'"),
            LogicErrorKind::AlreadyDeclaredModel { scope, model } => write!(f, "Model '{model}' is already declared in '{scope}'"),
            LogicErrorKind::UndeclaredTreatment { scope, treatment } => write!(f, "Treatment '{treatment}' is not declared in '{scope}'"),
            LogicErrorKind::AlreadyDeclaredTreatment { scope, treatment } => write!(f, "Treatment '{treatment}' is already declared in '{scope}'"),
            LogicErrorKind::UnexistingConnectionType { scope, from, output, to, input, output_type, input_type, output_flow, input_flow } => write!(f, "Connection from '{from}' to '{to}' in '{scope}' is not possible, '{output}' is {output_flow}<{output_type}> but '{input}' is {input_flow}<{input_type}>"),
            LogicErrorKind::UnsatisfiedOutput { scope, output } => write!(f, "Output '{output}' is not satisfied in '{scope}'"),
            LogicErrorKind::OverloadedOutput { scope, output } => write!(f, "Output '{output}' is overloaded in '{scope}', only one connection is possible to 'Self' outputs"),
            LogicErrorKind::UnmatchingModelType { scope: _, called, name, expected, given_name, given } => write!(f, "Model '{name}' for '{called}' is expected to be '{expected}', but given '{given_name}' is '{given}' and not based on it"),
            LogicErrorKind::UnexistingParametricModel { scope, called, parametric_model } => write!(f, "Parametric model '{parametric_model}' does not exist for '{called}' in '{scope}'"),
            LogicErrorKind::UnsetModel { scope, called, parametric_model } => write!(f, "No model assigned to '{parametric_model}' of '{called}' in '{scope}'"),
            LogicErrorKind::AlreadyIncludedBuildStep { treatment , cause_step,check_steps}  => write!(f, "Treatment '{treatment}' is referring to same instanciation of itself, causing infinite connection loop ({cause_step} already present in {})", check_steps.iter().map(|cs| cs.to_string()).collect::<Vec<_>>().join(", ")),
            LogicErrorKind::UnsatisfiedInput { scope, treatment, input } => if let Some(id) = scope {write!(f, "Input '{input}' of '{treatment}' is not satisfied in '{id}'")} else {write!(f, "Entrypoint have input '{input}' that must be satisfied")},
            LogicErrorKind::ConstRequiredVarProvided { scope, called, parameter, variable } => write!(f, "Parameter '{parameter}' of '{called}' is constant but provided '{variable}' is variable in '{scope}'"),
            LogicErrorKind::ConstRequiredContextProvided { scope, called, parameter, context, entry: _ } => write!(f, "Parameter '{parameter}' of '{called}' is constant but context '{context}' is provided in '{scope}', contexts are implicitly variable"),
            LogicErrorKind::ModelInstanciationConstOnly { scope, called, name, parameter } => write!(f, "Variable provided for parameter '{parameter}' of model '{name}' from type '{called}' in '{scope}', model instanciations can only get constants"),
            LogicErrorKind::ConstRequiredFunctionReturnsVar { scope, called, parameter, function } => write!(f, "Parameter '{parameter}' of '{called}' is constant but provided '{function}' have variable return value in '{scope}'"),
            LogicErrorKind::UnmatchingNumberOfParameters { scope, function } => write!(f, "Number of parameters given do not match for function '{function}' in '{scope}'"),
            LogicErrorKind::UnexistingGeneric { scope, element, name, described_type: _ } => write!(f, "The generic type '{name}' doesn't exist for '{element}' in '{scope}'"),
            LogicErrorKind::UndefinedGeneric { scope, element, described_type } => write!(f, "Generic '{described_type}' is not defined for '{element}' in '{scope}'"),
            LogicErrorKind::UnsatisfiedTraits { scope, element, described_type, unsatisfied_traits } => write!(f, "Type '{described_type}' does not satisfy trait {} for '{element}' in '{scope}'", unsatisfied_traits.iter().map(|tr| tr.to_string()).collect::<Vec<_>>().join(" + ")),
        }
    }
}

/// Handles and describe a Mélodium logic error.
#[derive(Debug, Clone)]
pub struct LogicError {
    /// Identifier of error.
    pub id: u32,
    /// Kind of error.
    pub kind: LogicErrorKind,
    /// Optional design reference attached to error.
    pub design_reference: Option<Arc<dyn Reference>>,
}

impl LogicError {
    /// Generates a new error with [`LogicErrorKind::CollectionUndefined`] kind.
    pub fn collection_undefined(id: u32, design_reference: Option<Arc<dyn Reference>>) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::CollectionUndefined,
        }
    }

    /// Generates a new error with [`LogicErrorKind::UncommitedDescriptor`] kind.
    pub fn uncommited_descriptor(
        id: u32,
        identifier: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UncommitedDescriptor { identifier },
        }
    }

    /// Generates a new error with [`LogicErrorKind::NoDesigner`] kind.
    pub fn no_designer(
        id: u32,
        identifier: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::NoDesigner { identifier },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ErroneousDesign`] kind.
    pub fn erroneous_design(
        id: u32,
        identifier: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ErroneousDesign { identifier },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ErroneousChecks`] kind.
    pub fn erroneous_checks(id: u32, design_reference: Option<Arc<dyn Reference>>) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ErroneousChecks,
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnavailableDesign`] kind.
    pub fn unavailable_design(
        id: u32,
        identifier: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnavailableDesign { identifier },
        }
    }

    /// Generates a new error with [`LogicErrorKind::LaunchExpectTreatment`] kind.
    pub fn launch_expect_treatment(id: u32, wrong_identifier: Identifier) -> Self {
        Self {
            id,
            design_reference: None,
            kind: LogicErrorKind::LaunchExpectTreatment { wrong_identifier },
        }
    }

    pub fn launch_wrong_parameter(id: u32, parameter: String) -> Self {
        Self {
            id,
            design_reference: None,
            kind: LogicErrorKind::LaunchWrongParameter { parameter },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingVariable`] kind.
    pub fn unexisting_variable(
        id: u32,
        identifier: Identifier,
        parameter: String,
        variable: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingVariable {
                identifier,
                parameter,
                variable,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingContextVariable`] kind.
    pub fn unexisting_context_variable(
        id: u32,
        identifier: Identifier,
        parameter: String,
        context: Identifier,
        variable: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingContextVariable {
                identifier,
                parameter,
                context,
                variable,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingParameter`] kind.
    pub fn unexisting_parameter(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parameter: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingParameter {
                scope,
                called,
                parameter,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnmatchingDataType`] kind.
    pub fn unmatching_datatype(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parameter: String,
        value: Value,
        expected: DescribedType,
        given: DescribedType,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnmatchingDataType {
                scope,
                called,
                parameter,
                value,
                expected,
                given,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnsetParameter`] kind.
    pub fn unset_parameter(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parameter: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnsetParameter {
                scope,
                called,
                parameter,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::MultipleParameterAssignation`] kind.
    pub fn multiple_parameter_assignation(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parameter: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::MultipleParameterAssignation {
                scope,
                called,
                parameter,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::NoValue`] kind.
    pub fn no_value(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parameter: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::NoValue {
                scope,
                called,
                parameter,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::NoContext`] kind.
    pub fn no_context(
        id: u32,
        scope: Identifier,
        model: Identifier,
        name: String,
        parameter: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::NoContext {
                scope,
                model,
                name,
                parameter,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnavailableContext`] kind.
    pub fn unavailable_context(
        id: u32,
        scope: Identifier,
        context: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnavailableContext { scope, context },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionInputNotFound`] kind.
    pub fn connection_input_not_found(
        id: u32,
        scope: Identifier,
        to: Identifier,
        input: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ConnectionInputNotFound { scope, to, input },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionSelfInputNotFound`] kind.
    pub fn connection_self_input_not_found(
        id: u32,
        scope: Identifier,
        input: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ConnectionSelfInputNotFound { scope, input },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionOutputNotFound`] kind.
    pub fn connection_output_not_found(
        id: u32,
        scope: Identifier,
        from: Identifier,
        output: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ConnectionOutputNotFound {
                scope,
                from,
                output,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConnectionSelfOutputNotFound`] kind.
    pub fn connection_self_output_not_found(
        id: u32,
        scope: Identifier,
        output: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ConnectionSelfOutputNotFound { scope, output },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingTreatment`] kind.
    pub fn unexisting_treatment(
        id: u32,
        scope: Identifier,
        claimed: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingTreatment { scope, claimed },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingModel`] kind.
    pub fn unexisting_model(
        id: u32,
        scope: Identifier,
        claimed: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingModel { scope, claimed },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingContext`] kind.
    pub fn unexisting_context(
        id: u32,
        scope: Identifier,
        claimed: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingContext { scope, claimed },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingFunction`] kind.
    pub fn unexisting_function(
        id: u32,
        scope: Identifier,
        claimed: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingFunction { scope, claimed },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UndeclaredModel`] kind.
    pub fn undeclared_model(
        id: u32,
        scope: Identifier,
        model: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UndeclaredModel { scope, model },
        }
    }

    /// Generates a new error with [`LogicErrorKind::AlreadyDeclaredModel`] kind.
    pub fn already_declared_model(
        id: u32,
        scope: Identifier,
        model: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::AlreadyDeclaredModel { scope, model },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UndeclaredTreatment`] kind.
    pub fn undeclared_treatment(
        id: u32,
        scope: Identifier,
        treatment: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UndeclaredTreatment { scope, treatment },
        }
    }

    /// Generates a new error with [`LogicErrorKind::AlreadyDeclaredTreatment`] kind.
    pub fn already_declared_treatment(
        id: u32,
        scope: Identifier,
        treatment: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::AlreadyDeclaredTreatment { scope, treatment },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingConnectionType`] kind.
    pub fn unexisting_connexion_type(
        id: u32,
        scope: Identifier,
        from: String,
        output: String,
        to: String,
        input: String,
        output_flow: Flow,
        output_type: DescribedType,
        input_flow: Flow,
        input_type: DescribedType,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingConnectionType {
                scope,
                from,
                output,
                to,
                input,
                output_flow,
                output_type,
                input_flow,
                input_type,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnsatisfiedOutput`] kind.
    pub fn unsatisfied_output(
        id: u32,
        scope: Identifier,
        output: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnsatisfiedOutput { scope, output },
        }
    }

    /// Generates a new error with [`LogicErrorKind::OverloadedOutput`] kind.
    pub fn overloaded_output(
        id: u32,
        scope: Identifier,
        output: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::OverloadedOutput { scope, output },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnmatchingModelType`] kind.
    pub fn unmatching_model_type(
        id: u32,
        scope: Identifier,
        called: Identifier,
        name: String,
        expected: Identifier,
        given_name: String,
        given: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnmatchingModelType {
                scope,
                called,
                name,
                expected,
                given_name,
                given,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingParametricModel`] kind.
    pub fn unexisting_parametric_model(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parametric_model: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingParametricModel {
                scope,
                called,
                parametric_model,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnsetModel`] kind.
    pub fn unset_model(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parametric_model: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnsetModel {
                scope,
                called,
                parametric_model,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::AlreadyIncludedBuildStep`] kind.
    pub fn already_included_build_step(
        id: u32,
        treatment: Identifier,
        cause_step: CheckStep,
        check_steps: Vec<CheckStep>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::AlreadyIncludedBuildStep {
                treatment,
                cause_step,
                check_steps,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnsatisfiedInput`] kind.
    pub fn unsatisfied_input(
        id: u32,
        scope: Option<Identifier>,
        treatment: String,
        input: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnsatisfiedInput {
                scope,
                treatment,
                input,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConstRequiredVarProvided`] kind.
    pub fn const_required_var_provided(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parameter: String,
        variable: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ConstRequiredVarProvided {
                scope,
                called,
                parameter,
                variable,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConstRequiredContextProvided`] kind.
    pub fn const_required_context_provided(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parameter: String,
        context: Identifier,
        entry: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ConstRequiredContextProvided {
                scope,
                called,
                parameter,
                context,
                entry,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ModelInstanciationConstOnly`] kind.
    pub fn model_instanciation_const_only(
        id: u32,
        scope: Identifier,
        called: Identifier,
        name: String,
        parameter: String,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ModelInstanciationConstOnly {
                scope,
                called,
                name,
                parameter,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::ConstRequiredFunctionReturnsVar`] kind.
    pub fn const_required_function_returns_var(
        id: u32,
        scope: Identifier,
        called: Identifier,
        parameter: String,
        function: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::ConstRequiredFunctionReturnsVar {
                scope,
                called,
                parameter,
                function,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnmatchingNumberOfParameters`] kind.
    pub fn unmatching_number_of_parameters(
        id: u32,
        scope: Identifier,
        function: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnmatchingNumberOfParameters { scope, function },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnexistingGeneric`] kind.
    pub fn unexisting_generic(
        id: u32,
        scope: Identifier,
        element: Identifier,
        name: String,
        described_type: DescribedType,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnexistingGeneric {
                scope,
                element,
                name,
                described_type,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UndefinedGeneric`] kind.
    pub fn undefined_generic(
        id: u32,
        scope: Identifier,
        element: Identifier,
        described_type: DescribedType,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UndefinedGeneric {
                scope,
                element,
                described_type,
            },
        }
    }

    /// Generates a new error with [`LogicErrorKind::UnsatisfiedTraits`] kind.
    pub fn unsatisfied_traits(
        id: u32,
        scope: Identifier,
        element: Identifier,
        described_type: DescribedType,
        unsatisfied_traits: Vec<DataTrait>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            id,
            design_reference,
            kind: LogicErrorKind::UnsatisfiedTraits {
                scope,
                element,
                described_type,
                unsatisfied_traits,
            },
        }
    }
}

impl Display for LogicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "D{:04}: {}", self.id, self.kind)
    }
}

pub type LogicErrors = Vec<LogicError>;
pub type LogicResult<T> = Status<T, LogicError, LogicError>;

impl From<LogicError> for LogicErrors {
    fn from(value: LogicError) -> Self {
        vec![value]
    }
}
