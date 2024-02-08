//! Provides restitution of Mélodium scripts from collection of elements.

pub mod area;
pub mod model;
pub mod treatment;
mod value;

pub use area::Area;
pub use model::Model;
pub use treatment::Treatment;

use melodium_common::descriptor::{DescribedType, Identifier};
use std::collections::BTreeMap;

fn describe_type(described_type: &DescribedType, names: &BTreeMap<Identifier, String>) -> String {
    match described_type {
        DescribedType::Vec(dt) => format!("Vec<{}>", describe_type(dt, names)),
        DescribedType::Option(dt) => format!("Option<{}>", describe_type(dt, names)),
        DescribedType::Data(data) => names
            .get(data.identifier())
            .cloned()
            .unwrap_or_else(|| data.identifier().name().to_string()),
        dt => dt.to_string(),
    }
}
