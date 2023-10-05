use melodium_common::descriptor::{
    Documented, Identified, Identifier, Model as ModelDescriptor, Parameterized,
};
use melodium_engine::design::Model as ModelDesign;
use std::collections::HashMap;

use super::value::value;
pub struct Model {
    design: ModelDesign,
    uses: Vec<Identifier>,
}

impl Model {
    pub fn new(design: ModelDesign) -> Self {
        let descriptor = design.descriptor.upgrade().unwrap();

        let uses = vec![descriptor.base_model().unwrap().identifier().clone()];
        Self { design, uses }
    }

    pub fn design(&self) -> &ModelDesign {
        &self.design
    }

    pub fn uses(&self) -> &Vec<Identifier> {
        &self.uses
    }

    pub fn implementation(&self, names: &HashMap<Identifier, String>) -> String {
        let descriptor = self.design.descriptor.upgrade().unwrap();

        let mut implementation = format!(
            "/**\n{}*/\n",
            descriptor
                .documentation()
                .lines()
                .map(|l| format!("\t{l}"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        implementation.push_str("model ");
        implementation.push_str(descriptor.identifier().name());

        implementation.push_str("(");

        implementation.push_str(
            &descriptor
                .parameters()
                .iter()
                .map(|(_, param)| param.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );

        implementation.push_str(")\n{\n");

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
