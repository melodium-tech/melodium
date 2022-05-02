
use std::sync::{Arc, RwLock};
use crate::script::path::Path;
use crate::script::semantic::{
    model::Model,
    declared_model::DeclaredModel,
    declared_parameter::DeclaredParameter,
    sequence::Sequence,
    r#type::{Type, TypeStructure},
};

pub fn model(model: &Model, path: &Path) -> String {

    let parameters = if !model.parameters.is_empty() {
        let mut string = String::new();

        for param in &model.parameters {
            string.push_str(&format!("↳ `{}`  \n", &parameter(&param.read().unwrap())));
        }

        format!("#### Parameters\n\n{}", string)
    }
    else { String::default() };

    format!("# Model {}\n\n`{}`\n\nBased on `{}`\n\n---\n\n{}\n\n---\n\n{}", model.name, path.to_identifier(&model.name).unwrap(), model.text.r#type.string, parameters, model.text.doc.clone().unwrap_or_default().string)
}

pub fn sequence(sequence: &Sequence, path: &Path) -> String {

    let exposed_models: Vec<Arc<RwLock<DeclaredModel>>> =
        sequence.declared_models.iter().filter(|dm| dm.read().unwrap().text.is_some()).map(|dm| Arc::clone(dm)).collect();
    let models = if !exposed_models.is_empty() {
        let mut string = String::new();

        for model in &sequence.declared_models {
            string.push_str(&format!("⊔ `{}`  \n", &declared_model(&model.read().unwrap())));
        }

        format!("#### Configuration\n\n{}", string)
    }
    else { String::default() };

    let parameters = if !sequence.parameters.is_empty() {
        let mut string = String::new();

        for param in &sequence.parameters {
            string.push_str(&format!("↳ `{}`  \n", &parameter(&param.read().unwrap())));
        }

        format!("#### Parameters\n\n{}", string)
    }
    else { String::default() };

    let requirements = if !sequence.requirements.is_empty() {
        let mut string = String::new();

        for req in &sequence.requirements {
            string.push_str(&format!("○ `{}`  \n", &req.read().unwrap().name));
        }

        format!("#### Require\n\n{}", string)
    }
    else { String::default() };

    let inputs = if !sequence.inputs.is_empty() {
        let mut string = String::new();

        for input in &sequence.inputs {
            let input = input.read().unwrap();
            string.push_str(&format!("⇥ `{}: {}`  \n", &input.name, io_type(&input.r#type)));
        }

        format!("#### Inputs\n\n{}", string)
    }
    else { String::default() };

    let outputs = if !sequence.outputs.is_empty() {
        let mut string = String::new();

        for output in &sequence.outputs {
            let output = output.read().unwrap();
            string.push_str(&format!("↦ `{}: {}`  \n", &output.name, io_type(&output.r#type)));
        }

        format!("#### Outputs\n\n{}", string)
    }
    else { String::default() };

    format!("# Sequence {}\n\n`{}`\n\n---\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n---\n\n{}\n\n", sequence.name, path.to_identifier(&sequence.name).unwrap(), models, parameters, requirements, inputs, outputs, sequence.text.doc.clone().unwrap_or_default().string)
}

pub fn parameter(parameter: &DeclaredParameter) -> String {

    format!("{} {}: {} {}", parameter.variability, parameter.name, param_type(&parameter.r#type),
        if let Some(def) = &parameter.value {
            if let Ok(val) = def.read().unwrap().content.make_executive_value(&parameter.r#type.make_descriptor().unwrap().0) {
                format!("= {}", val)
            }
            else { String::default() }
        }
        else { String::default() }
    )
}

pub fn io_type(t: &Type) -> String {

    let stru = match t.structure {
        TypeStructure::Scalar => t.name.to_string(),
        TypeStructure::Vector => format!("{}<{}>", t.structure, t.name),
    };

    format!("{}<{}>", t.flow, stru)
}

pub fn param_type(t: &Type) -> String {

    match t.structure {
        TypeStructure::Scalar => t.name.to_string(),
        TypeStructure::Vector => format!("{}<{}>", t.structure, t.name),
    }
}

pub fn declared_model(dm: &DeclaredModel) -> String {
    
    if let Some(text) = &dm.text {
        format!("{}: {}", dm.name, text.r#type.as_ref().unwrap().name.string)
    }
    else { String::default() }
}

