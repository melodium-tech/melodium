use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadBookChapterRequest {
    /// Chapter path as reported by `search_book`, e.g.
    /// `programming/elements/functions.md`.
    pub path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ReadBookChapterResult {
    pub found: bool,
    pub error: Option<String>,
    /// Raw Markdown content of the chapter, when found.
    pub content: Option<String>,
}

pub fn read_book_chapter(request: ReadBookChapterRequest) -> ReadBookChapterResult {
    match crate::book::read_chapter(&request.path) {
        Some(content) => ReadBookChapterResult {
            found: true,
            error: None,
            content: Some(content.to_string()),
        },
        None => ReadBookChapterResult {
            found: false,
            error: Some(format!(
                "no book chapter at '{}' (use search_book to list valid paths)",
                request.path
            )),
            content: None,
        },
    }
}
