use super::value::value;
use melodium_common::descriptor::{
    Identified, Identifier, Parameterized, Treatment as TreatmentDescriptor,
};
use melodium_engine::design::{Connection, Treatment as TreatmentDesign, IO};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Treatment {
    design: Arc<TreatmentDesign>,
}

impl Treatment {
    pub fn new(design: Arc<TreatmentDesign>) -> Self {
        Self { design }
    }

    pub fn uses(&self) -> Vec<Identifier> {
        let mut uses = Vec::new();

        let descriptor = self.design.descriptor.upgrade().unwrap();

        for (_, model) in descriptor.models() {
            uses.push(model.identifier().clone())
        }

        for (_, context) in descriptor.contexts() {
            uses.push(context.identifier().clone())
        }

        for (_, model) in &self.design.model_instanciations {
            uses.push(model.descriptor.upgrade().unwrap().identifier().clone())
        }

        for (_, treatment) in &self.design.treatments {
            uses.push(treatment.descriptor.upgrade().unwrap().identifier().clone())
        }

        uses
    }

    pub fn implementation(&self, names: &HashMap<Identifier, String>) -> String {
        let descriptor = self.design.descriptor.upgrade().unwrap();

        let mut implementation = String::new();

        implementation.push_str("treatment ");
        implementation.push_str(descriptor.identifier().name());

        if !descriptor.models().is_empty() {
            implementation.push_str("[");

            for (name, model) in descriptor.models() {
                implementation.push_str(name);
                implementation.push_str(": ");
                implementation.push_str(names.get(model.identifier()).unwrap());
                implementation.push_str(", ");
            }
            implementation.truncate(implementation.len() - 2);

            implementation.push_str("]");
        }

        implementation.push_str("(");

        for (_, param) in descriptor.parameters() {
            implementation.push_str(&param.to_string());
            implementation.push_str(", ");
        }
        implementation.truncate(implementation.len() - 2);

        implementation.push_str(")\n");

        for (_, context) in descriptor.contexts() {
            implementation.push_str("  require ");
            implementation.push_str(names.get(context.identifier()).unwrap());
            implementation.push_str("\n");
        }

        for (_, input) in descriptor.inputs() {
            implementation.push_str("  input ");
            implementation.push_str(&input.to_string());
            implementation.push_str("\n");
        }

        for (_, output) in descriptor.outputs() {
            implementation.push_str("  output ");
            implementation.push_str(&output.to_string());
            implementation.push_str("\n");
        }

        for (_, model) in &self.design.model_instanciations {
            implementation.push_str("  model ");
            implementation.push_str(&model.name);
            implementation.push_str(": ");
            implementation.push_str(
                names
                    .get(model.descriptor.upgrade().unwrap().identifier())
                    .unwrap(),
            );

            implementation.push_str("(");

            for (_, param) in &model.parameters {
                implementation.push_str(&param.name);
                implementation.push_str(" = ");
                implementation.push_str(&value(&param.value, names));
                implementation.push_str(", ");
            }
            implementation.truncate(implementation.len() - 2);

            implementation.push_str(")\n");
        }

        implementation.push_str("{");

        for (_, instanciation) in &self.design.treatments {
            implementation.push_str("    ");
            implementation.push_str(&instanciation.name);

            let treatment_name = names
                .get(instanciation.descriptor.upgrade().unwrap().identifier())
                .unwrap();
            if treatment_name != &instanciation.name {
                implementation.push_str(": ");
                implementation.push_str(treatment_name);
            }

            if !instanciation.models.is_empty() {
                implementation.push_str("[");
                for (name, model) in &instanciation.models {
                    implementation.push_str(name);
                    implementation.push_str(" = ");
                    implementation.push_str(model);
                    implementation.push_str(", ");
                }
                implementation.truncate(implementation.len() - 2);
                implementation.push_str("]");
            }

            implementation.push_str("(");

            for (_, param) in &instanciation.parameters {
                implementation.push_str(&param.name);
                implementation.push_str(" = ");
                implementation.push_str(&value(&param.value, names));
                implementation.push_str(", ");
            }
            implementation.truncate(implementation.len() - 2);

            implementation.push_str(")\n");
        }

        implementation.push_str("\n");

        for connection in &self.design.connections {
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
