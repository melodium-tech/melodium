
use std::sync::{Arc, RwLock};
use crate::logic::designer::{SequenceDesigner, ConnectionDesigner, ConnectionIODesigner, ModelInstanciationDesigner, TreatmentDesigner, ValueDesigner};
use crate::logic::descriptor::*;

use super::script::Uses;

pub struct Sequence {
    designer: Arc<RwLock<SequenceDesigner>>
}

impl Sequence {

    pub fn new(designer: &Arc<RwLock<SequenceDesigner>>) -> Self {
        Self {
            designer: Arc::clone(designer)
        }
    }

    pub fn uses(&self) -> Vec<IdentifierDescriptor> {

        let mut uses = Vec::new();
        let designer = self.designer.read().unwrap();

        fn get_func(uses: &mut Vec<IdentifierDescriptor>, val: &ValueDesigner) {
            match val {
                ValueDesigner::Function(desc, vals) => {
                    uses.push(desc.identifier().clone());
                    vals.iter().for_each(|v| get_func(uses, v));
                },
                _ => {}
            }
        }

        // Treatments & their functions
        for (_, treatment) in designer.treatments() {

            let treatment = treatment.read().unwrap();
            uses.push(treatment.descriptor().identifier().clone());
            treatment.parameters().iter().for_each(|(_, p)| get_func(&mut uses, p.read().unwrap().value().as_ref().unwrap()));
        }

        // Models 
        for (_, model) in designer.descriptor().models() {

            uses.push(model.identifier().clone());
        }

        // Instanciations & their functions
        for (_, model) in designer.model_instanciations() {

            let model = model.read().unwrap();
            uses.push(model.descriptor().identifier().clone());
            model.parameters().iter().for_each(|(_, p)| get_func(&mut uses, p.read().unwrap().value().as_ref().unwrap()));
        }

        uses
    }

    pub fn generate(&self, uses: &Uses) -> String {

        let mut result = String::new();
        let designer = self.designer.read().unwrap();
        let descriptor = designer.descriptor();

        result.push_str(&format!("sequence {}", descriptor.identifier().name()));

        if !descriptor.models().is_empty() {

            result.push_str("[");
            for (name, model) in descriptor.models() {
                result.push_str(&format!("{} {}", name, uses.get(model.identifier())))
            }
            result.push_str("]");
        }

        result.push_str("(");
        result.push_str(&descriptor.parameters()
            .iter()
            .map(
                |(_, param)| super::declared_parameter::declared_parameter(param)
            )
            .collect::<Vec<_>>()
            .join(", "));
        result.push_str(")\n");

        for (_, requirement) in descriptor.requirements() {
            result.push_str(&format!("  require {}\n", requirement.name()));
        }

        for (_, input) in descriptor.inputs() {
            result.push_str(&format!("  input {}\n", Self::input(input)));
        }

        for (_, output) in descriptor.outputs() {
            result.push_str(&format!("  output {}\n", Self::output(output)));
        }

        for (_, model_instanciation) in designer.model_instanciations() {
            result.push_str(&format!("  model {}\n", Self::model_instanciation(uses, &model_instanciation.read().unwrap())));
        }

        result.push_str("{\n\n");

        for (_, treatment) in designer.treatments() {
            result.push_str(&format!("    {}\n", Self::treatment(uses, &treatment.read().unwrap())));
        }

        result.push_str("\n");

        for connection in designer.connections() {
            result.push_str(&format!("    {}\n", Self::connection(&connection.read().unwrap())));
        }

        result.push_str("}\n");

        result
    }

    fn input(input: &InputDescriptor) -> String {

        let result = input.datatype().to_string();

        let result = match input.flow() {
            FlowDescriptor::Block => format!("Block<{}>", result),
            FlowDescriptor::Stream => format!("Stream<{}>", result),
        };

        format!("{}: {}", input.name(), result)
    }

    fn output(output: &OutputDescriptor) -> String {

        let result = output.datatype().to_string();

        let result = match output.flow() {
            FlowDescriptor::Block => format!("Block<{}>", result),
            FlowDescriptor::Stream => format!("Stream<{}>", result),
        };

        format!("{}: {}", output.name(), result)
    }

    fn model_instanciation(uses: &Uses, mi: &ModelInstanciationDesigner) -> String {

        let mut result = format!("{}: {}(", mi.name(), uses.get(mi.descriptor().identifier()));

        result.push_str(&mi.parameters()
            .iter()
            .map(
                |(_, param)| super::assigned_parameter::assigned_parameter(uses, &param.read().unwrap())
            )
            .collect::<Vec<_>>()
            .join(", ")
        );
        
        result.push_str(")");

        result
    }

    fn treatment(uses: &Uses, treatment: &TreatmentDesigner) -> String {

        format!("{}({})",
            if treatment.name() == uses.get(treatment.descriptor().identifier()) { treatment.name().to_string() }
            else {
                format!("{}: {}", treatment.name(), uses.get(treatment.descriptor().identifier()))
            },
            treatment.parameters()
            .iter()
            .map(
                |(_, param)| super::assigned_parameter::assigned_parameter(uses, &param.read().unwrap())
            )
            .collect::<Vec<_>>()
            .join(", ")
        )

    }

    fn connection(connection: &ConnectionDesigner) -> String {

        format!("{}.{} -> {}.{}",
            match connection.output_treatment().as_ref().unwrap() {
                ConnectionIODesigner::Sequence() => "Self".to_string(),
                ConnectionIODesigner::Treatment(t) => {
                    t.upgrade().unwrap().read().unwrap().name().to_string()
                }
            },
            connection.output_name().as_ref().unwrap(),
            match connection.input_treatment().as_ref().unwrap() {
                ConnectionIODesigner::Sequence() => "Self".to_string(),
                ConnectionIODesigner::Treatment(t) => {
                    t.upgrade().unwrap().read().unwrap().name().to_string()
                }
            },
            connection.input_name().as_ref().unwrap()
        )
    }
}

