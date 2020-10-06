
//! Module dedicated to DeclarativeElement trait definition.

use std::rc::Rc;
use std::cell::RefCell;

use super::common::Node;
use super::declared_parameter::DeclaredParameter;
use super::model::Model;
use super::sequence::Sequence;

/// Trait for elements that are declarative blocks.
/// 
/// A declarative element is a block which owns declared parameters.
pub trait DeclarativeElement : Node {

    /// Returns a reference on the structure.
    fn declarative_element(&self) -> DeclarativeElementType;

    /// Search for a declared parameter.
    fn find_declared_parameter(&self, name: & str) -> Option<&Rc<RefCell<DeclaredParameter>>>;
}

/// Enum listing possible declarative elements.
pub enum DeclarativeElementType<'a> {
    Model(&'a Model),
    Sequence(&'a Sequence)
}
