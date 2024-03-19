//! Provides Mélodium script error management.
//!
//! The main type of this module is [ScriptError], which handles most of the management, combined with kind of errors detailed with [ScriptErrorKind].

use crate::text::Kind;
use crate::text::PositionnedString;
use crate::text::Word;

use melodium_common::descriptor::Status;
use melodium_engine::LogicError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// Handles and describe a Mélodium script error.
///
/// Most of the properties are deeply related with [Word](super::text::word::Word).
///
/// # Note
/// All positions (`absolute_position`, `line_position`) are expected to be bytes indexes, not chars.
#[derive(Debug, Clone)]
pub struct ScriptError {
    /// Identifier of error.
    pub id: u32,
    /// Kind of error.
    pub kind: ScriptErrorKind,
}

/// Kind of script error that might happens.
#[derive(Debug, Clone)]
pub enum ScriptErrorKind {
    /// The error is related to a specific word that disable script to work.
    Word {
        word: Word,
        expected: &'static [Kind],
    },
    /// The error is about an unexcpected end of script.
    EndOfScript,
    DescriptionElementExpected {
        word: Word,
        element: Word,
        expected: &'static [&'static str],
    },
    DeclarationExpected {
        word: Word,
        expected: &'static [&'static str],
    },
    InvalidRoot {
        text: PositionnedString,
        root: String,
    },
    AlreadyUsedName {
        text: PositionnedString,
    },
    InvalidType {
        text: PositionnedString,
    },
    InvalidStructure {
        text: PositionnedString,
    },
    UnimportedElement {
        text: PositionnedString,
    },
    AlreadyDeclared {
        text: PositionnedString,
    },
    AlreadyAssigned {
        text: PositionnedString,
    },
    MissingType {
        text: PositionnedString,
    },
    MissingValue {
        text: PositionnedString,
    },
    DefaultForbidden {
        text: PositionnedString,
    },
    ConstDeclarationOnly {
        text: PositionnedString,
    },
    FlowForbidden {
        text: PositionnedString,
    },
    StructureForbidden {
        text: PositionnedString,
    },
    TypeForbidden {
        text: PositionnedString,
    },
    ConnectionMustTransmit {
        from: PositionnedString,
        to: PositionnedString,
    },
    TreatmentNotFound {
        text: PositionnedString,
    },
    NameRequired {
        text: PositionnedString,
    },
    UndeclaredModel {
        text: PositionnedString,
    },
    UndeclaredParameter {
        text: PositionnedString,
    },
    UndeclaredContext {
        text: PositionnedString,
    },
    UndeclaredData {
        text: PositionnedString,
    },
    ReferenceUnset {
        debug_reference: String,
    },
    InvalidBoolean {
        text: PositionnedString,
    },
    InvalidNumber {
        text: PositionnedString,
    },
    InvalidString {
        text: PositionnedString,
    },
    InvalidCharacter {
        text: PositionnedString,
    },
    InvalidByte {
        text: PositionnedString,
    },
    ExecutiveRestitutionFailed {
        text: PositionnedString,
        message: String,
    },
    MissingFunctionParameter {
        text: PositionnedString,
        index: usize,
    },
    MissingFunctionGeneric {
        text: PositionnedString,
        index: usize,
    },
    MissingTreatmentGeneric {
        text: PositionnedString,
    },
    InvalidGeneric {
        text: PositionnedString,
    },
    InvalidTrait {
        text: PositionnedString,
    },
    UnexistingDependency {
        text: PositionnedString,
        root: String,
    },
    /// The error comes from logic.
    Logic {
        error: LogicError,
    },
    /// No descriptor associated
    NoDescriptor {
        name: PositionnedString,
    },
}

impl Display for ScriptErrorKind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ScriptErrorKind::Word { word, expected } => write!(
                f,
                "found '{}' at line {} position {}, expecting {}",
                word.text, word.position.line_number, word.position.line_position, expected.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(", ")
            ),
            ScriptErrorKind::EndOfScript => write!(f, "reached unexpected end of script"),
            ScriptErrorKind::DescriptionElementExpected {
                word,
                element,
                expected,
            } => write!(
                f,
                "for '{}' at line {} position {}, '{}' is not an expected element; {} are admitted",
                element.text,
                word.position.line_number,
                word.position.line_position,
                word.text,
                expected
                    .iter()
                    .map(|s| format!("'{s}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ScriptErrorKind::DeclarationExpected {
                word,
                expected,
            } => write!(
                f,
                "found '{}' at line {} position {} while a declaration is expected; {} are admitted",
                word.text,
                word.position.line_number,
                word.position.line_position,
                expected
                    .iter()
                    .map(|s| format!("'{s}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ScriptErrorKind::InvalidRoot { text, root } => write!(f, "at line {} position {} '{root}' is not a valid root", text.position.line_number, text.position.line_position),
            ScriptErrorKind::AlreadyUsedName { text } => write!(f, "at line {} position {} '{}' is already used as name", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::InvalidType { text } => write!(f, "at line {} position {} '{}' is not a valid type", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::InvalidStructure { text } => write!(f, "at line {} position {} '{}' is not a valid structure", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::UnimportedElement { text } => write!(f, "at line {} position {} element '{}' is not imported nor locally available", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::AlreadyDeclared { text } => write!(f, "at line {} position {} '{}' is already declared", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::AlreadyAssigned { text } => write!(f, "at line {} position {} '{}' is already assigned", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::MissingType { text } => write!(f, "at line {} position {} type for '{}' is missing", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::MissingValue { text } => write!(f, "at line {} position {} value for '{}' is missing", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::DefaultForbidden { text } => write!(f, "at line {} position {} '{}' cannot have default value", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::ConstDeclarationOnly { text } => write!(f, "at line {} position {} '{}' cannot be otherwise than 'const'", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::FlowForbidden { text } => write!(f, "at line {} position {} '{}' cannot have flow specification", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::StructureForbidden { text } => write!(f, "at line {} position {} '{}' cannot have structure specification", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::TypeForbidden { text } => write!(f, "at line {} position {} '{}' cannot have type specification", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::ConnectionMustTransmit { from, to } => write!(f, "at line {} position {}, connection from '{}' to '{}' must transmit data", from.position.line_number, from.position.line_position, from.string, to.string),
            ScriptErrorKind::TreatmentNotFound { text } => write!(f, "at line {} position {} cannot find treatment '{}'", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::NameRequired { text } => write!(f, "at line {} position {} a name is required for assignation to '{}'", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::UndeclaredModel { text } => write!(f, "at line {} position {} no model declared as '{}'", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::UndeclaredParameter { text } => write!(f, "at line {} position {} no parameter declared as '{}'", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::UndeclaredContext { text } => write!(f, "at line {} position {} no context declared as '{}'", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::UndeclaredData { text } => write!(f, "at line {} position {} no data type declared as '{}'", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::ReferenceUnset { debug_reference } => write!(f, "reference is not set, this is an internal error: '{debug_reference}'"),
            ScriptErrorKind::InvalidBoolean { text } => write!(f, "at line {} position {} '{}' is not a boolean value", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::InvalidNumber { text } => write!(f, "at line {} position {} '{}' is not a numeric value", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::InvalidString { text } => write!(f, "at line {} position {} '{}' is not a correctly formatted string", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::InvalidCharacter { text } => write!(f, "at line {} position {} '{}' is not a valid character", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::InvalidByte { text } => write!(f, "at line {} position {} '{}' is not a valid byte", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::ExecutiveRestitutionFailed { text, message } => write!(f, "at line {} position {} error occured with value '{}' because: {}", text.position.line_number, text.position.line_position, text.string, message),
            ScriptErrorKind::MissingFunctionParameter { text, index } => write!(f, "at line {} position {} for function '{}' parameter is missing at position {index}", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::MissingFunctionGeneric { text, index } => write!(f, "at line {} position {} for function '{}' generic is missing at position {index}", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::MissingTreatmentGeneric { text } => write!(f, "at line {} position {} for treatment '{}' generic is missing", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::InvalidGeneric { text } => write!(f, "at line {} position {} '{}' is not a valid generic name", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::InvalidTrait { text } => write!(f, "at line {} position {} '{}' is not a valid trait", text.position.line_number, text.position.line_position, text.string),
            ScriptErrorKind::UnexistingDependency { text, root } => write!(f, "at line {} position {} '{root}' is not a dependency", text.position.line_number, text.position.line_position),
            ScriptErrorKind::Logic { error } => {
                if let Some(ps) = error
                    .design_reference
                    .as_ref().and_then(|ptr| Arc::clone(ptr).downcast_arc::<PositionnedString>().ok())
                {
                    write!(
                        f,
                        "at line {} position {} '{}': {}",
                        ps.position.line_number, ps.position.line_position, ps.string, error
                    )
                } else {
                    write!(f, "{}", error)
                }
            }
            ScriptErrorKind::NoDescriptor { name } => write!(
                f,
                "no descriptor available for '{}' line {} position {}, this is an internal error",
                name.string, name.position.line_number, name.position.line_position
            ),
        }
    }
}

impl ScriptError {
    /// Creates a new error of Word kind.
    ///
    /// The ScriptError created that way will be of [ScriptErrorKind::Word] kind.
    /// Each parameter matches the properties of ScriptError.
    pub fn word(id: u32, word: Word, expected: &'static [Kind]) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::Word { word, expected },
        }
    }

    /// Creates a new error of EndOfScript kind.
    ///
    /// The ScriptError created that way will be of [ScriptErrorKind::EndOfScript] kind.
    pub fn end_of_script(id: u32) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::EndOfScript,
        }
    }

    pub fn description_element_expected(
        id: u32,
        word: Word,
        element: Word,
        expected: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::DescriptionElementExpected {
                word,
                element,
                expected,
            },
        }
    }

    pub fn declaration_expected(id: u32, word: Word, expected: &'static [&'static str]) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::DeclarationExpected { word, expected },
        }
    }

    pub fn invalid_root(id: u32, text: PositionnedString, root: String) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidRoot { text, root },
        }
    }

    pub fn already_used_name(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::AlreadyUsedName { text },
        }
    }

    pub fn invalid_type(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidType { text },
        }
    }

    pub fn invalid_structure(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidStructure { text },
        }
    }

    pub fn unimported_element(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::UnimportedElement { text },
        }
    }

    pub fn already_declared(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::AlreadyDeclared { text },
        }
    }

    pub fn already_assigned(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::AlreadyAssigned { text },
        }
    }

    pub fn missing_type(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::MissingType { text },
        }
    }

    pub fn missing_value(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::MissingValue { text },
        }
    }

    pub fn default_forbidden(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::DefaultForbidden { text },
        }
    }

    pub fn const_declaration_only(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::ConstDeclarationOnly { text },
        }
    }

    pub fn flow_forbidden(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::FlowForbidden { text },
        }
    }

    pub fn structure_forbidden(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::StructureForbidden { text },
        }
    }

    pub fn type_forbidden(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::TypeForbidden { text },
        }
    }

    pub fn connection_must_transmit_data(
        id: u32,
        from: PositionnedString,
        to: PositionnedString,
    ) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::ConnectionMustTransmit { from, to },
        }
    }

    pub fn treatment_not_found(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::TreatmentNotFound { text },
        }
    }

    pub fn name_required(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::NameRequired { text },
        }
    }

    pub fn undeclared_model(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::UndeclaredModel { text },
        }
    }

    pub fn undeclared_parameter(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::UndeclaredParameter { text },
        }
    }

    pub fn undeclared_context(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::UndeclaredContext { text },
        }
    }

    pub fn undeclared_data(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::UndeclaredData { text },
        }
    }

    pub fn reference_unset(id: u32, debug_reference: String) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::ReferenceUnset { debug_reference },
        }
    }

    pub fn invalid_boolean(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidBoolean { text },
        }
    }

    pub fn invalid_number(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidNumber { text },
        }
    }

    pub fn invalid_string(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidString { text },
        }
    }

    pub fn invalid_character(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidCharacter { text },
        }
    }

    pub fn invalid_byte(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidByte { text },
        }
    }

    pub fn executive_restitution_failed(id: u32, text: PositionnedString, message: String) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::ExecutiveRestitutionFailed { text, message },
        }
    }

    pub fn missing_function_parameter(id: u32, text: PositionnedString, index: usize) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::MissingFunctionParameter { text, index },
        }
    }

    pub fn missing_function_generic(id: u32, text: PositionnedString, index: usize) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::MissingFunctionGeneric { text, index },
        }
    }

    pub fn missing_treatment_generic(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::MissingTreatmentGeneric { text },
        }
    }

    pub fn invalid_generic(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidGeneric { text },
        }
    }

    pub fn invalid_trait(id: u32, text: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::InvalidTrait { text },
        }
    }

    pub fn unexisting_dependency(id: u32, text: PositionnedString, root: String) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::UnexistingDependency { text, root },
        }
    }

    pub fn logic(id: u32, error: LogicError) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::Logic { error },
        }
    }

    pub fn no_descriptor(id: u32, name: PositionnedString) -> Self {
        Self {
            id,
            kind: ScriptErrorKind::NoDescriptor { name },
        }
    }
}

impl Display for ScriptError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "S{:04}: {}", self.id, self.kind)
    }
}

impl From<LogicError> for ScriptError {
    fn from(le: LogicError) -> Self {
        ScriptError::logic(0, le)
    }
}

impl Error for ScriptError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub type ScriptErrors = Vec<ScriptError>;
pub type ScriptResult<T> = Status<T, ScriptError, ScriptError>;

impl From<ScriptError> for ScriptErrors {
    fn from(value: ScriptError) -> Self {
        vec![value]
    }
}
