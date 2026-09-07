use crate::docs::CicdSource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCicdMigrationGuideRequest {
    /// CI system to migrate from.
    pub source: CicdSource,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GetCicdMigrationGuideResult {
    /// Reference guide covering how to migrate pipelines from the given CI
    /// system to Mélodium's `cicd` package, in Markdown.
    pub guide: String,
}

pub fn get_cicd_migration_guide(
    request: GetCicdMigrationGuideRequest,
) -> GetCicdMigrationGuideResult {
    GetCicdMigrationGuideResult {
        guide: crate::docs::cicd_migration_guide(request.source).to_string(),
    }
}
