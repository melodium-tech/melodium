//! Provides restitution of Mélodium scripts from collection of elements.

mod area;
mod layout;
mod model;
mod treatment;
mod value;

pub use area::Area;
pub use model::Model;
pub use treatment::Treatment;

use melodium_common::descriptor::{Attribute, Attributes, DescribedType, Identifier};
use std::collections::BTreeMap;

/// Options of restitution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestitutionOptions {
    /// Width lines are kept within, unless a single element is longer on its own.
    pub width: usize,
}

impl Default for RestitutionOptions {
    fn default() -> Self {
        Self { width: 100 }
    }
}

fn sorted_attributes(attributes: &Attributes) -> Vec<(&String, &Attribute)> {
    let mut attributes = attributes.iter().collect::<Vec<_>>();
    attributes.sort();
    attributes
}

fn describe_type(described_type: &DescribedType, names: &BTreeMap<Identifier, String>) -> String {
    match described_type {
        DescribedType::Vec(dt) => format!("Vec<{}>", describe_type(dt, names)),
        DescribedType::Option(dt) => format!("Option<{}>", describe_type(dt, names)),
        DescribedType::Secret(dt) => format!("Secret<{}>", describe_type(dt, names)),
        DescribedType::Generic(generic) => generic.name.clone(),
        DescribedType::Data(data) => names
            .get(data.identifier())
            .cloned()
            .unwrap_or_else(|| data.identifier().name().to_string()),
        dt => dt.to_string(),
    }
}
