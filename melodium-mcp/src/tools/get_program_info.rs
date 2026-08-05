use crate::tools::Diagnostic;
use melodium::LoadingConfig;
use melodium_common::descriptor::{Entry, IdentifierRequirement};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetProgramInfoRequest {
    /// Program file to inspect, can be either a `.mel`, `Compo.toml`, or `.jeu` file.
    pub path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ParameterInfo {
    pub name: String,
    /// `const` or `var`.
    pub variability: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub default: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct EntrypointInfo {
    pub command: String,
    pub identifier: String,
    pub documentation: String,
    pub parameters: Vec<ParameterInfo>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GetProgramInfoResult {
    pub success: bool,
    pub errors: Vec<Diagnostic>,
    pub entrypoints: Vec<EntrypointInfo>,
}

pub fn get_program_info(request: GetProgramInfoRequest) -> GetProgramInfoResult {
    let file = PathBuf::from(&request.path);
    let result = melodium::load_file_all_entrypoints(file, LoadingConfig::new());

    let errors = result
        .failure()
        .into_iter()
        .map(Diagnostic::from)
        .chain(result.errors().iter().map(Diagnostic::from))
        .collect();

    let entrypoints = result
        .success()
        .map(|(pkg, collection)| {
            pkg.entrypoints()
                .iter()
                .filter_map(|(command, identifier)| {
                    let entry = collection.get(&IdentifierRequirement::from(identifier))?;
                    let Entry::Treatment(treatment) = entry else {
                        return None;
                    };
                    let mut parameters = treatment
                        .parameters()
                        .values()
                        .map(|parameter| ParameterInfo {
                            name: parameter.name().to_string(),
                            variability: parameter.variability().to_string(),
                            type_: parameter.described_type().to_string(),
                            default: parameter.default().as_ref().map(|v| v.to_string()),
                        })
                        .collect::<Vec<_>>();
                    parameters.sort_by(|a, b| a.name.cmp(&b.name));

                    Some(EntrypointInfo {
                        command: command.clone(),
                        identifier: treatment.identifier().to_string(),
                        documentation: treatment.documentation().to_string(),
                        parameters,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    GetProgramInfoResult {
        success: result.is_success(),
        errors,
        entrypoints,
    }
}
