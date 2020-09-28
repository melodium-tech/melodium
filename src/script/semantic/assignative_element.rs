
//! Module dedicated to AssignativeElement trait definition.

use std::rc::Rc;
use std::cell::RefCell;

use super::common::Node;
use super::assigned_parameter::AssignedParameter;
use super::model::Model;
use super::treatment::Treatment;

/// Trait for elements that are assignative blocks or components.
/// 
/// An assignative element is a block or component which assign value to parameters.
/// An assignative element also always represent a subsitute for the declarative element
/// it belongs to.
pub trait AssignativeElement : Node {

    /// Returns a reference on the structure.
    fn assignative_element(&self) -> AssignativeElementType;

    /// Search for an assigned parameter.
    fn find_assigned_parameter(&self, name: & str) -> Option<&Rc<RefCell<AssignedParameter>>>;
}

/// Enum listing possible declarative elements.
pub enum AssignativeElementType<'a> {
    Model(&'a Model),
    Treatment(&'a Treatment)
}
