use super::{
    describe_type,
    layout::{layout, Link, Node},
    sorted_attributes,
    value::{inline_value, value},
    RestitutionOptions,
};
use itertools::Itertools;
use melodium_common::descriptor::{
    Attribuable, Documented, Generics, Identified, Identifier, Parameterized,
    Treatment as TreatmentDescriptor,
};
use melodium_engine::design::{Treatment as TreatmentDesign, IO};
use std::collections::BTreeMap;

pub struct Treatment {
    design: TreatmentDesign,
    uses: Vec<Identifier>,
}

impl Treatment {
    pub fn new(design: TreatmentDesign) -> Self {
        let mut uses = design.descriptor.upgrade().unwrap().uses();

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
                        "{attributes}\n    {variability} {name}: {param}{default}",
                        variability = param.variability(),
                        attributes = param
                            .attributes()
                            .iter()
                            .sorted()
                            .map(|(name, attribute)| format!("\n    #[{name}({attribute})]"))
                            .collect::<Vec<_>>()
                            .join(""),
                        name = param.name(),
                        param = describe_type(param.described_type(), names),
                        default = param
                            .default()
                            .as_ref()
                            .map(|v| format!(" = {}", value(&v.into(), names, 1)))
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        );

        if !descriptor.parameters().is_empty() {
            implementation.push_str("\n");
        }

        implementation.push_str(")\n");

        for (_, context) in descriptor.contexts().iter().sorted_by_key(|(k, _)| *k) {
            implementation.push_str("  require ");
            implementation.push_str(names.get(context.identifier()).unwrap());
            implementation.push_str("\n");
        }

        for (_, input) in descriptor.inputs().iter().sorted_by_key(|(k, _)| *k) {
            for (name, attribute) in sorted_attributes(input.attributes()) {
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
            for (name, attribute) in sorted_attributes(output.attributes()) {
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
            for (name, attribute) in sorted_attributes(model.attributes()) {
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

            let parameters = |value: &dyn Fn(&_) -> String| {
                model
                    .parameters
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .map(|(_, param)| {
                        format!("{name} = {}", value(&param.value), name = param.name)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let single_line = parameters(&|v| inline_value(v, names));
            if !single_line.contains('\n')
                && implementation
                    .lines()
                    .last()
                    .unwrap_or_default()
                    .chars()
                    .count()
                    + single_line.chars().count()
                    + 1
                    <= options.width
            {
                implementation.push_str(&single_line);
            } else {
                implementation.push_str(&parameters(&|v| value(v, names, 1)));
            }

            implementation.push_str(")\n");
        }

        implementation.push_str("{\n");

        let connections = layout(
            self.design
                .connections
                .iter()
                .map(|connection| Link {
                    source: match &connection.output_treatment {
                        IO::Sequence() => Node::Inputs,
                        IO::Treatment(name) => Node::Instance(name.clone()),
                    },
                    output: connection.output_name.clone(),
                    destination: match &connection.input_treatment {
                        IO::Sequence() => Node::Outputs,
                        IO::Treatment(name) => Node::Instance(name.clone()),
                    },
                    input: connection.input_name.clone(),
                    attributes: sorted_attributes(connection.attributes())
                        .into_iter()
                        .map(|(name, attribute)| (name.clone(), attribute.clone()))
                        .collect(),
                })
                .collect(),
            4,
            options.width,
        );

        // Instanciations are declared in the order connections show them, then by name.
        let instanciations = connections
            .order
            .iter()
            .filter_map(|name| self.design.treatments.get(name))
            .chain(
                self.design
                    .treatments
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .filter(|(k, _)| !connections.order.contains(k))
                    .map(|(_, instanciation)| instanciation),
            );

        for instanciation in instanciations {
            let descriptor = instanciation.descriptor.upgrade().unwrap();

            for (name, attribute) in sorted_attributes(instanciation.attributes()) {
                implementation.push_str("    #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }

            let mut declaration = format!("    {}", instanciation.name);

            let treatment_name = names.get(descriptor.identifier()).unwrap();
            if treatment_name != &instanciation.name {
                declaration.push_str(": ");
                declaration.push_str(treatment_name);
            }

            if !descriptor.generics().is_empty() && !instanciation.generics.is_empty() {
                declaration.push('<');

                declaration.push_str(
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

                declaration.push('>');
            }

            if !instanciation.models.is_empty() {
                declaration.push_str("[");
                declaration.push_str(
                    &instanciation
                        .models
                        .iter()
                        .sorted_by_key(|(k, _)| *k)
                        .map(|(name, model)| format!("{name} = {model}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                declaration.push_str("]");
            }

            // Parameters stay on the line of the declaration when they fit in it.
            let parameters = instanciation
                .parameters
                .iter()
                .sorted_by_key(|(k, _)| *k)
                .map(|(_, param)| (&param.name, &param.value))
                .collect::<Vec<_>>();
            let single_line = format!(
                "{declaration}({})",
                parameters
                    .iter()
                    .map(|(name, value)| format!("{name} = {}", inline_value(value, names)))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if parameters.is_empty()
                || (!single_line.contains('\n') && single_line.chars().count() <= options.width)
            {
                implementation.push_str(&single_line);
            } else {
                implementation.push_str(&declaration);
                implementation.push_str("(");
                implementation.push_str(
                    &parameters
                        .iter()
                        .map(|(name, param_value)| {
                            format!(
                                "\n        {name} = {value}",
                                value = value(param_value, names, 3)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(","),
                );
                implementation.push_str("\n    )");
            }
            implementation.push_str("\n");
        }

        if !connections.lines.is_empty() && !self.design.treatments.is_empty() {
            implementation.push_str("\n");
        }

        for line in connections.lines {
            implementation.push_str(&line);
            implementation.push_str("\n");
        }

        implementation.push_str("}\n\n");

        implementation
    }
}
