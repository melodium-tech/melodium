use super::{
    sorted_attributes,
    value::{inline_value, value},
    RestitutionOptions,
};
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
        let mut uses = design.descriptor.upgrade().unwrap().uses();

        uses.retain(|id| id != design.descriptor.upgrade().unwrap().identifier());

        Self { design, uses }
    }

    pub fn design(&self) -> &ModelDesign {
        &self.design
    }

    pub fn uses(&self) -> &Vec<Identifier> {
        &self.uses
    }

    pub fn implementation(&self, names: &BTreeMap<Identifier, String>) -> String {
        self.implementation_with_options(names, &RestitutionOptions::default())
    }

    pub fn implementation_with_options(
        &self,
        names: &BTreeMap<Identifier, String>,
        options: &RestitutionOptions,
    ) -> String {
        let descriptor = self.design.descriptor.upgrade().unwrap();

        let mut implementation = if descriptor.documentation().trim().is_empty() {
            String::new()
        } else {
            format!(
                "/**\n{}\n*/\n",
                descriptor
                    .documentation()
                    .lines()
                    .map(|l| format!("\t{l}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        for (name, attribute) in sorted_attributes(descriptor.attributes()) {
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
                        "{attributes}\n    const {name}: {param}{default}",
                        attributes = param
                            .attributes()
                            .iter()
                            .sorted()
                            .map(|(name, attribute)| format!("\n    #[{name}({attribute})] "))
                            .collect::<Vec<_>>()
                            .join(""),
                        name = param.name(),
                        param = describe_type(param.described_type(), names),
                        default = param
                            .default()
                            .as_ref()
                            .map(|v| format!(" = {}", value(&v.into(), names, 2)))
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        );

        if !descriptor.parameters().is_empty() {
            implementation.push_str("\n");
        }

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
            let single_line = inline_value(&param.value, names);
            if !single_line.contains('\n')
                && param.name.chars().count() + single_line.chars().count() + 7 <= options.width
            {
                implementation.push_str(&single_line);
            } else {
                implementation.push_str(&value(&param.value, names, 2));
            }
            implementation.push_str("\n");
        }

        implementation.push_str("}\n\n");

        implementation
    }
}
