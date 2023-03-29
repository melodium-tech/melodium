use melodium_common::descriptor::{
    Identified, Identifier, Model as ModelDescriptor, Parameterized,
};
use melodium_engine::design::Model as ModelDesign;
use std::collections::HashMap;
use std::sync::Arc;

use super::value::value;
pub struct Model {
    design: Arc<ModelDesign>,
}

impl Model {
    pub fn new(design: Arc<ModelDesign>) -> Self {
        Self { design }
    }

    pub fn uses(&self) -> Vec<Identifier> {
        let descriptor = self.design.descriptor.upgrade().unwrap();

        vec![descriptor.base_model().unwrap().identifier().clone()]
    }

    pub fn implementation(&self, names: &HashMap<Identifier, String>) -> String {
        let descriptor = self.design.descriptor.upgrade().unwrap();

        let mut implementation = String::new();

        implementation.push_str("model ");
        implementation.push_str(descriptor.identifier().name());

        implementation.push_str("(");

        for (_, param) in descriptor.parameters() {
            implementation.push_str(&param.to_string());
            implementation.push_str(", ");
        }
        implementation.truncate(implementation.len() - 2);

        implementation.push_str(")\n{");

        for (_, param) in &self.design.parameters {
            implementation.push_str("    ");
            implementation.push_str(&param.name);
            implementation.push_str(" = ");
            implementation.push_str(&value(&param.value, names));
            implementation.push_str("\n");
        }

        implementation.push_str("}\n\n");

        implementation
    }
}
