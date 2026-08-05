use melodium::LoadingConfig;
use melodium_common::descriptor::{Collection, Identifier, LoadingError, LoadingResult};
use melodium_loader::PackageInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, path::PathBuf, sync::Arc};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CheckProgramRequest {
    /// Program file to check, can be either a `.mel`, `Compo.toml`, or `.jeu` file.
    pub path: String,
    /// Entrypoint command to check (defaults to `main`). Ignored if `all` is set.
    pub entrypoint: Option<String>,
    /// Force a specific identifier (e.g. `my_package/area::treatment`) to be used as entrypoint,
    /// instead of one declared through the package entrypoints. Ignored if `all` is set.
    pub force_identifier: Option<String>,
    /// Check every element reachable from the required packages, ignoring entrypoints.
    pub all: Option<bool>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Diagnostic {
    pub id: u32,
    pub message: String,
}

impl From<&LoadingError> for Diagnostic {
    fn from(error: &LoadingError) -> Self {
        Self {
            id: error.id,
            message: error.kind.to_string(),
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CheckProgramResult {
    pub success: bool,
    pub errors: Vec<Diagnostic>,
    pub entrypoints: Vec<String>,
}

pub fn check_program(request: CheckProgramRequest) -> CheckProgramResult {
    // `melodium::load_file`/`load_compo`/`load_raw` already extend the config
    // with `core_config()` internally, so `core_packages` must stay empty here
    // to avoid loading every core package twice.
    let config = LoadingConfig::new();

    let file = PathBuf::from(&request.path);
    let all = request.all.unwrap_or(false);

    let result = if all {
        melodium::load_file_all_entrypoints(file, config)
    } else if let Some(identifier) = request.force_identifier.as_deref() {
        match Identifier::try_from(identifier) {
            Ok(identifier) => melodium::load_file_force_entrypoint(file, &identifier, config),
            Err(_) => {
                return CheckProgramResult {
                    success: false,
                    errors: vec![Diagnostic {
                        id: 0,
                        message: format!("'{identifier}' is not a valid identifier"),
                    }],
                    entrypoints: Vec::new(),
                }
            }
        }
    } else {
        let entrypoint = request.entrypoint.as_deref().unwrap_or("main");
        melodium::load_file(file, entrypoint, config)
    };

    to_check_result(&result)
}

fn to_check_result(
    result: &LoadingResult<(Arc<dyn PackageInfo>, Arc<Collection>)>,
) -> CheckProgramResult {
    let entrypoints = result
        .success()
        .map(|(pkg, _)| pkg.entrypoints().keys().cloned().collect())
        .unwrap_or_default();

    let errors = result
        .failure()
        .into_iter()
        .map(Diagnostic::from)
        .chain(result.errors().iter().map(Diagnostic::from))
        .collect();

    CheckProgramResult {
        success: result.is_success(),
        errors,
        entrypoints,
    }
}
