//! Module dedicated to AssignativeElement trait definition.

use super::assigned_model::AssignedModel;
use super::assigned_parameter::AssignedParameter;
use super::common::Node;
use super::declarative_element::DeclarativeElement;
use super::model::Model;
use super::model_instanciation::ModelInstanciation;
use super::treatment_instanciation::TreatmentInstanciation;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

/// Trait for elements that are assignative blocks or components.
///
/// An assignative element is a block or component which assign value to parameters.
/// An assignative element also always represent a subsitute for the declarative element
/// it belongs to.
pub trait AssignativeElement: Node + Debug + Send + Sync {
    /// Returns a reference on the structure.
    fn assignative_element(&'_ self) -> AssignativeElementType<'_>;

    /// Returns the associated declarative element.
    ///
    /// This gives access to what the assignation might be if referring to declared items.
    fn associated_declarative_element(&self) -> Arc<RwLock<dyn DeclarativeElement>>;

    /// Search for an assigned model.
    fn find_assigned_model(&self, _name: &str) -> Option<&Arc<RwLock<AssignedModel>>> {
        None
    }

    /// Search for an assigned parameter.
    fn find_assigned_parameter(&self, name: &str) -> Option<&Arc<RwLock<AssignedParameter>>>;
}

/// Enum listing possible declarative elements.
#[derive(Debug)]
pub enum AssignativeElementType<'a> {
    Model(&'a Model),
    ModelInstanciation(&'a ModelInstanciation),
    Treatment(&'a TreatmentInstanciation),
}
