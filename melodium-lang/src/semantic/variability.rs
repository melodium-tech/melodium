//! Module for Variability semantic analysis.

use melodium_common::descriptor::Variability as VariabilityDescriptor;
use std::fmt;

/// Enum for variability identification.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Variability {
    /// Variability is constant.
    Const,
    /// Variability is variable.
    Var,
}

impl Variability {
    pub fn from_string(text: &str) -> Option<Self> {
        match text {
            "const" => Some(Self::Const),
            "var" => Some(Self::Var),
            _ => None,
        }
    }

    pub fn to_descriptor(&self) -> VariabilityDescriptor {
        match self {
            Self::Const => VariabilityDescriptor::Const,
            Self::Var => VariabilityDescriptor::Var,
        }
    }
}

impl fmt::Display for Variability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Variability::Const => "const",
                Variability::Var => "var",
            }
        )
    }
}
