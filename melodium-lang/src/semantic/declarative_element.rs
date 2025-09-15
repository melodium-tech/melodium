//! Module dedicated to DeclarativeElement trait definition.

use super::common::Node;
use super::declared_parameter::DeclaredParameter;
use super::model::Model;
use super::treatment::Treatment;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// Trait for elements that are declarative blocks.
///
/// A declarative element is a block which owns declared parameters.
pub trait DeclarativeElement: Node + Debug + Send + Sync {
    /// Returns a reference on the structure.
    fn declarative_element(&'_ self) -> DeclarativeElementType<'_>;

    /// Search for a declared parameter.
    fn find_declared_parameter(&self, name: &str) -> Option<&Arc<RwLock<DeclaredParameter>>>;
}

/// Enum listing possible declarative elements.
#[derive(Debug)]
pub enum DeclarativeElementType<'a> {
    Model(&'a Model),
    Treatment(&'a Treatment),
    None,
}

/// Provides a 'none' declarative element, useful for raw value and functions parsing.
#[derive(Debug)]
pub struct NoneDeclarativeElement;

impl DeclarativeElement for NoneDeclarativeElement {
    fn declarative_element(&'_ self) -> DeclarativeElementType<'_> {
        DeclarativeElementType::None
    }

    fn find_declared_parameter(&self, _name: &str) -> Option<&Arc<RwLock<DeclaredParameter>>> {
        None
    }
}

impl Node for NoneDeclarativeElement {}
