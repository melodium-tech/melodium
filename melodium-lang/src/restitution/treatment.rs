use super::{describe_type, value::value};
use itertools::Itertools;
use melodium_common::descriptor::{
    Attribuable, Documented, Generics, Identified, Identifier, Parameterized,
    Treatment as TreatmentDescriptor,
};
use melodium_engine::design::{Connection, Treatment as TreatmentDesign, IO};
use std::collections::BTreeMap;

pub struct Treatment {
    design: TreatmentDesign,
    uses: Vec<Identifier>,
}

impl Treatment {
    pub fn new(design: TreatmentDesign) -> Self {
        let mut uses = design.uses();

        uses.retain(|id| id != design.descriptor.upgrade().unwrap().identifier());

        Self { design, uses }
    }

    pub fn design(&self) -> &TreatmentDesign {
        &self.design
    }

    pub fn uses(&self) -> &Vec<Identifier> {
        &self.uses
    }

    pub fn implementation(&self, names: &BTreeMap<Identifier, String>) -> String {
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

        for (name, attribute) in descriptor.attributes() {
            implementation.push_str("#[");
            implementation.push_str(name);
            implementation.push_str("(");
            implementation.push_str(&attribute);
            implementation.push_str(")]\n");
        }

        implementation.push_str("treatment ");
        implementation.push_str(descriptor.identifier().name());

        if !descriptor.generics().is_empty() {
            implementation.push('<');

            implementation.push_str(
                &descriptor
                    .generics()
                    .iter()
                    .map(|generic| {
                        if generic.traits.is_empty() {
                            generic.name.clone()
                        } else {
                            format!(
                                "{}: {}",
                                generic.name,
                                generic
                                    .traits
                                    .iter()
                                    .map(|tr| tr.to_string())
                                    .collect::<Vec<_>>()
                                    .join(" + ")
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );

            implementation.push('>');
        }

        if !descriptor.models().is_empty() {
            implementation.push_str("[");

            implementation.push_str(
                &descriptor
                    .models()
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .map(|(name, model)| {
                        format!("{name}: {id}", id = names.get(model.identifier()).unwrap())
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );

            implementation.push_str("]");
        }

        implementation.push_str("(");

        implementation.push_str(
            &descriptor
                .parameters()
                .iter()
                .sorted_by_key(|(k, _)| *k)
                .map(|(_, param)| {
                    format!(
                        "{attributes}\n        {variability} {name}: {param}{default}",
                        variability = param.variability(),
                        attributes = param
                            .attributes()
                            .iter()
                            .map(|(name, attribute)| format!("\n        #[{name}({attribute})]"))
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

        implementation.push_str(")\n");

        for (_, context) in descriptor.contexts().iter().sorted_by_key(|(k, _)| *k) {
            implementation.push_str("  require ");
            implementation.push_str(names.get(context.identifier()).unwrap());
            implementation.push_str("\n");
        }

        for (_, input) in descriptor.inputs().iter().sorted_by_key(|(k, _)| *k) {
            for (name, attribute) in input.attributes() {
                implementation.push_str("  #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }
            implementation.push_str("  input ");
            implementation.push_str(input.name());
            implementation.push_str(": ");
            implementation.push_str(&input.flow().to_string());
            implementation.push_str("<");
            implementation.push_str(&describe_type(input.described_type(), names));
            implementation.push_str(">\n");
        }

        for (_, output) in descriptor.outputs().iter().sorted_by_key(|(k, _)| *k) {
            for (name, attribute) in output.attributes() {
                implementation.push_str("  #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }
            implementation.push_str("  output ");
            implementation.push_str(output.name());
            implementation.push_str(": ");
            implementation.push_str(&output.flow().to_string());
            implementation.push_str("<");
            implementation.push_str(&describe_type(output.described_type(), names));
            implementation.push_str(">\n");
        }

        for (_, model) in self
            .design
            .model_instanciations
            .iter()
            .sorted_by_key(|(k, _)| *k)
        {
            for (name, attribute) in model.attributes() {
                implementation.push_str("  #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }
            implementation.push_str("  model ");
            implementation.push_str(&model.name);
            implementation.push_str(": ");
            implementation.push_str(
                names
                    .get(model.descriptor.upgrade().unwrap().identifier())
                    .unwrap(),
            );

            implementation.push_str("(");

            implementation.push_str(
                &model
                    .parameters
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .map(|(_, param)| {
                        format!(
                            "{name} = {value}",
                            name = param.name,
                            value = value(&param.value, names, 1)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );

            implementation.push_str(")\n");
        }

        implementation.push_str("{\n");

        for (_, instanciation) in self.design.treatments.iter().sorted_by_key(|(k, _)| *k) {
            let descriptor = instanciation.descriptor.upgrade().unwrap();

            for (name, attribute) in instanciation.attributes() {
                implementation.push_str("    #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }
            implementation.push_str("    ");
            implementation.push_str(&instanciation.name);

            let treatment_name = names.get(descriptor.identifier()).unwrap();
            if treatment_name != &instanciation.name {
                implementation.push_str(": ");
                implementation.push_str(treatment_name);
            }

            if !descriptor.generics().is_empty() && !instanciation.generics.is_empty() {
                implementation.push('<');

                implementation.push_str(
                    &descriptor
                        .generics()
                        .iter()
                        .map(|generic| {
                            instanciation
                                .generics
                                .get(&generic.name)
                                .map(|desc_type| describe_type(desc_type, names))
                                .unwrap_or_else(|| "_".to_string())
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                );

                implementation.push('>');
            }

            if !instanciation.models.is_empty() {
                implementation.push_str("[");
                implementation.push_str(
                    &instanciation
                        .models
                        .iter()
                        .sorted_by_key(|(k, _)| *k)
                        .map(|(name, model)| format!("{name} = {model}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                implementation.push_str("]");
            }

            implementation.push_str("(");

            implementation.push_str(
                &instanciation
                    .parameters
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .map(|(_, param)| {
                        format!(
                            "\n        {name} = {value}",
                            name = param.name,
                            value = value(&param.value, names, 2)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(","),
            );

            implementation.push_str(")\n");
        }

        implementation.push_str("\n");

        for connection in &self.design.connections {
            for (name, attribute) in connection.attributes() {
                implementation.push_str("    #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }
            implementation.push_str("    ");
            implementation.push_str(&Self::connection(connection));
            implementation.push_str("\n");
        }

        implementation.push_str("}\n\n");

        implementation
    }

    fn connection(connection: &Connection) -> String {
        format!(
            "{source}.{output} -> {receiver}.{input}",
            source = Self::io(&connection.output_treatment),
            output = connection.output_name,
            receiver = Self::io(&connection.input_treatment),
            input = connection.input_name,
        )
    }

    fn io(io: &IO) -> &str {
        match io {
            IO::Sequence() => "Self",
            IO::Treatment(name) => name,
        }
    }
}
