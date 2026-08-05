use crate::reference::{entry_documentation, entry_identifier_string, entry_kind, summary_line};
use melodium_common::descriptor::{Collection, IdentifierRequirement};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_RESULTS: usize = 300;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListLibraryElementsRequest {
    /// Restrict to elements whose area starts with this `/`-separated prefix
    /// (e.g. `std/flow`, `http`, `sql`). Omit to list every loaded package.
    pub area: Option<String>,
    /// Restrict to one kind: `treatment`, `function`, `model`, `context`, or `data`.
    pub kind: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ElementSummary {
    pub identifier: String,
    pub kind: String,
    pub summary: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListLibraryElementsResult {
    pub elements: Vec<ElementSummary>,
    /// True if more elements matched the filter than were returned.
    pub truncated: bool,
}

pub fn list_library_elements(
    request: ListLibraryElementsRequest,
    collection: &Collection,
) -> ListLibraryElementsResult {
    let area_segments: Option<Vec<String>> = request
        .area
        .as_deref()
        .map(|area| area.split('/').map(str::to_string).collect());
    let kind_filter = request.kind.as_deref().map(str::to_lowercase);

    let mut identifiers = collection.identifiers();
    identifiers.sort();

    let mut matches = Vec::new();
    let mut truncated = false;

    for identifier in identifiers {
        if let Some(segments) = &area_segments {
            if !identifier.path().starts_with(segments.as_slice()) {
                continue;
            }
        }

        let Some(entry) = collection.get(&IdentifierRequirement::from(&identifier)) else {
            continue;
        };

        let kind = entry_kind(entry);
        if let Some(filter) = &kind_filter {
            if kind != filter {
                continue;
            }
        }

        if matches.len() >= MAX_RESULTS {
            truncated = true;
            break;
        }

        matches.push(ElementSummary {
            identifier: entry_identifier_string(entry),
            kind: kind.to_string(),
            summary: summary_line(entry_documentation(entry)),
        });
    }

    ListLibraryElementsResult {
        elements: matches,
        truncated,
    }
}
