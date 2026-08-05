use crate::tools::ParameterInfo;
use melodium_common::descriptor::{Collection, Entry, Identifier, IdentifierRequirement};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DescribeElementRequest {
    /// Fully-qualified identifier, e.g. `std/flow::emit` or `http/server::HttpServer`.
    pub identifier: String,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ElementDetail {
    Treatment {
        identifier: String,
        documentation: String,
        generics: Vec<String>,
        models: Vec<ModelRef>,
        contexts: Vec<String>,
        parameters: Vec<ParameterInfo>,
        inputs: Vec<String>,
        outputs: Vec<String>,
    },
    Function {
        identifier: String,
        documentation: String,
        generics: Vec<String>,
        parameters: Vec<ParameterInfo>,
        return_type: String,
    },
    Model {
        identifier: String,
        documentation: String,
        is_core_model: bool,
        base_model: Option<String>,
        parameters: Vec<ParameterInfo>,
    },
    Context {
        identifier: String,
        documentation: String,
        values: Vec<ContextValue>,
    },
    Data {
        identifier: String,
        documentation: String,
        signature: String,
    },
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ModelRef {
    pub name: String,
    pub identifier: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ContextValue {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DescribeElementResult {
    pub found: bool,
    pub error: Option<String>,
    pub element: Option<ElementDetail>,
}

fn parameters_of(
    parameters: impl Iterator<Item = (String, String, String, Option<String>)>,
) -> Vec<ParameterInfo> {
    let mut params: Vec<ParameterInfo> = parameters
        .map(|(name, variability, type_, default)| ParameterInfo {
            name,
            variability,
            type_,
            default,
        })
        .collect();
    params.sort_by(|a, b| a.name.cmp(&b.name));
    params
}

pub fn describe_element(
    request: DescribeElementRequest,
    collection: &Collection,
) -> DescribeElementResult {
    let identifier = match Identifier::try_from(request.identifier.as_str()) {
        Ok(identifier) => identifier,
        Err(_) => {
            return DescribeElementResult {
                found: false,
                error: Some(format!(
                    "'{}' is not a valid identifier (expected form: area/path::name)",
                    request.identifier
                )),
                element: None,
            }
        }
    };

    let Some(entry) = collection.get(&IdentifierRequirement::from(&identifier)) else {
        return DescribeElementResult {
            found: false,
            error: Some(format!(
                "no loaded element matches '{}'",
                request.identifier
            )),
            element: None,
        };
    };

    let element = match entry {
        Entry::Treatment(treatment) => ElementDetail::Treatment {
            identifier: treatment.identifier().to_string(),
            documentation: treatment.documentation().to_string(),
            generics: treatment.generics().iter().map(|g| g.to_string()).collect(),
            models: treatment
                .models()
                .iter()
                .map(|(name, model)| ModelRef {
                    name: name.clone(),
                    identifier: model.identifier().to_string(),
                })
                .collect(),
            contexts: treatment
                .contexts()
                .values()
                .map(|context| context.identifier().to_string())
                .collect(),
            parameters: parameters_of(treatment.parameters().values().map(|p| {
                (
                    p.name().to_string(),
                    p.variability().to_string(),
                    p.described_type().to_string(),
                    p.default().as_ref().map(|v| v.to_string()),
                )
            })),
            inputs: treatment.inputs().values().map(|i| i.to_string()).collect(),
            outputs: treatment
                .outputs()
                .values()
                .map(|o| o.to_string())
                .collect(),
        },
        Entry::Function(function) => ElementDetail::Function {
            identifier: function.identifier().to_string(),
            documentation: function.documentation().to_string(),
            generics: function.generics().iter().map(|g| g.to_string()).collect(),
            parameters: parameters_of(function.parameters().iter().map(|p| {
                (
                    p.name().to_string(),
                    p.variability().to_string(),
                    p.described_type().to_string(),
                    p.default().as_ref().map(|v| v.to_string()),
                )
            })),
            return_type: function.return_type().to_string(),
        },
        Entry::Model(model) => ElementDetail::Model {
            identifier: model.identifier().to_string(),
            documentation: model.documentation().to_string(),
            is_core_model: model.is_core_model(),
            base_model: model.base_model().map(|m| m.identifier().to_string()),
            parameters: parameters_of(model.parameters().values().map(|p| {
                (
                    p.name().to_string(),
                    p.variability().to_string(),
                    p.described_type().to_string(),
                    p.default().as_ref().map(|v| v.to_string()),
                )
            })),
        },
        Entry::Context(context) => ElementDetail::Context {
            identifier: context.identifier().to_string(),
            documentation: context.documentation().to_string(),
            values: context
                .values()
                .iter()
                .map(|(name, data_type)| ContextValue {
                    name: name.clone(),
                    type_: data_type.to_string(),
                })
                .collect(),
        },
        Entry::Data(data) => ElementDetail::Data {
            identifier: data.identifier().to_string(),
            documentation: data.documentation().to_string(),
            signature: data.to_string(),
        },
    };

    DescribeElementResult {
        found: true,
        error: None,
        element: Some(element),
    }
}
