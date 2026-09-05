use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLanguageGuideRequest {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GetLanguageGuideResult {
    /// Explanation of Mélodium's dataflow execution model (treatments,
    /// tracks, models, contexts, connections) aimed at an AI reading
    /// or writing Mélodium code, in Markdown.
    pub guide: String,
}

pub fn get_language_guide(_request: GetLanguageGuideRequest) -> GetLanguageGuideResult {
    GetLanguageGuideResult {
        guide: crate::docs::language_guide().to_string(),
    }
}
