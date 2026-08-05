use crate::reference::{entry_documentation, entry_identifier_string, entry_kind, summary_line};
use melodium_common::descriptor::{Collection, IdentifierRequirement};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 100;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchReferenceRequest {
    /// Case-insensitive text searched across identifiers and documentation.
    pub query: String,
    /// Maximum number of results to return (default 20, capped at 100).
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SearchMatch {
    pub identifier: String,
    pub kind: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SearchReferenceResult {
    pub matches: Vec<SearchMatch>,
    pub truncated: bool,
}

pub fn search_reference(
    request: SearchReferenceRequest,
    collection: &Collection,
) -> SearchReferenceResult {
    let limit = (request.limit.unwrap_or(DEFAULT_LIMIT as u32) as usize).min(MAX_LIMIT);
    let query = request.query.to_lowercase();

    let mut identifiers = collection.identifiers();
    identifiers.sort();

    let mut matches = Vec::new();
    let mut truncated = false;

    for identifier in identifiers {
        let Some(entry) = collection.get(&IdentifierRequirement::from(&identifier)) else {
            continue;
        };

        let identifier_string = entry_identifier_string(entry);
        let documentation = entry_documentation(entry);

        if !identifier_string.to_lowercase().contains(&query)
            && !documentation.to_lowercase().contains(&query)
        {
            continue;
        }

        if matches.len() >= limit {
            truncated = true;
            break;
        }

        matches.push(SearchMatch {
            identifier: identifier_string,
            kind: entry_kind(entry).to_string(),
            snippet: summary_line(documentation),
        });
    }

    SearchReferenceResult { matches, truncated }
}
