use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 100;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchBookRequest {
    /// Case-insensitive text searched across chapter paths, titles, and
    /// content. Omit or leave empty to list every chapter (table of
    /// contents) instead of searching.
    pub query: Option<String>,
    /// Maximum number of results to return (default 20, capped at 100).
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct BookMatch {
    /// Chapter path, to pass to `read_book_chapter`.
    pub path: String,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SearchBookResult {
    pub matches: Vec<BookMatch>,
    pub truncated: bool,
}

pub fn search_book(request: SearchBookRequest) -> SearchBookResult {
    let limit = (request.limit.unwrap_or(DEFAULT_LIMIT as u32) as usize).min(MAX_LIMIT);

    match request.query.filter(|q| !q.trim().is_empty()) {
        None => {
            let chapters = crate::book::chapters();
            let truncated = chapters.len() > limit;
            let matches = chapters
                .into_iter()
                .take(limit)
                .map(|c| BookMatch {
                    path: c.path,
                    title: c.title.clone(),
                    snippet: c.title,
                })
                .collect();
            SearchBookResult { matches, truncated }
        }
        Some(query) => {
            let (found, truncated) = crate::book::search(&query, limit);
            let matches = found
                .into_iter()
                .map(|m| BookMatch {
                    path: m.path,
                    title: m.title,
                    snippet: m.snippet,
                })
                .collect();
            SearchBookResult { matches, truncated }
        }
    }
}
