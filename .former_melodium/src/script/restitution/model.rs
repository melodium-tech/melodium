
use std::sync::{Arc, RwLock};
use crate::logic::designer::{ModelDesigner, ValueDesigner};
use crate::logic::descriptor::*;

use super::script::Uses;

pub struct Model {
    designer: Arc<RwLock<ModelDesigner>>
}

impl Model {
    pub fn new(designer: &Arc<RwLock<ModelDesigner>>) -> Self {
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

        uses.push(designer.descriptor().core_model().identifier().clone());

        designer.parameters().iter().for_each(|(_, p)| get_func(&mut uses, p.read().unwrap().value().as_ref().unwrap()));

        uses
    }

    pub fn generate(&self, uses: &Uses) -> String {

        let mut result = String::new();
        let designer = self.designer.read().unwrap();
        let descriptor = designer.descriptor();

        result.push_str(&format!("model {}", descriptor.identifier().name()));

        result.push_str("(");
        result.push_str(&descriptor.parameters()
            .iter()
            .map(
                |(_, param)| super::declared_parameter::declared_parameter(param)
            )
            .collect::<Vec<_>>()
            .join(", "));
        result.push_str(")\n{\n\n    \n");

        result.push_str(&designer.parameters()
            .iter()
            .map(
                |(_, param)| super::assigned_parameter::assigned_parameter(uses, &param.read().unwrap())
            )
            .collect::<Vec<_>>()
            .join("\n    "));

        result.push_str("\n}\n");

        result
    }

}
