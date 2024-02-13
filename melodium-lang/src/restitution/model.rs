use super::value::value;
use crate::restitution::describe_type;
use itertools::Itertools;
use melodium_common::descriptor::{
    Attribuable, Documented, Identified, Identifier, Model as ModelDescriptor, Parameterized,
};
use melodium_engine::design::Model as ModelDesign;
use std::collections::BTreeMap;
pub struct Model {
    design: ModelDesign,
    uses: Vec<Identifier>,
}

impl Model {
    pub fn new(design: ModelDesign) -> Self {
        let uses = design.uses();

        Self { design, uses }
    }

    pub fn design(&self) -> &ModelDesign {
        &self.design
    }

    pub fn uses(&self) -> &Vec<Identifier> {
        &self.uses
    }

    pub fn implementation(&self, names: &BTreeMap<Identifier, String>) -> String {
        let descriptor = self.design.descriptor.upgrade().unwrap();

        let mut implementation = if descriptor.documentation().is_empty() {
            String::new()
        } else {
            format!(
                "/**{}*/\n",
                descriptor
                    .documentation()
                    .lines()
                    .map(|l| format!("\t{l}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        for (name, attribute) in descriptor.attributes() {
            implementation.push_str("#[");
            implementation.push_str(name);
            implementation.push_str("(");
            implementation.push_str(&attribute);
            implementation.push_str(")]\n");
        }

        implementation.push_str("model ");
        implementation.push_str(descriptor.identifier().name());

        implementation.push_str("(");

        implementation.push_str(
            &descriptor
                .parameters()
                .iter()
                .sorted_by_key(|(k, _)| *k)
                .map(|(_, param)| {
                    format!(
                        "{attributes}{param}",
                        attributes = param
                            .attributes()
                            .iter()
                            .map(|(name, attribute)| format!("#[{name}({attribute})] "))
                            .collect::<Vec<_>>()
                            .join(""),
                        param = describe_type(param.described_type(), names),
                    )
                })
                .collect::<Vec<_>>()
                .join(", "),
        );

        implementation.push_str("): ");
        implementation.push_str(
            names
                .get(descriptor.base_model().unwrap().identifier())
                .unwrap(),
        );

        implementation.push_str("\n{\n");

        for (_, param) in self.design.parameters.iter().sorted_by_key(|(k, _)| *k) {
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
