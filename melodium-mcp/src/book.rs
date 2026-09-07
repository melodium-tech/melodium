//! The vendored Mélodium book (see `book/README.md` for provenance),
//! embedded at compile time so it can be browsed and searched without a
//! network dependency.

use include_dir::{include_dir, Dir, DirEntry, File};
use std::path::Path;

static BOOK: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/book");

pub struct Chapter {
    pub path: String,
    pub title: String,
}

pub struct BookMatch {
    pub path: String,
    pub title: String,
    pub snippet: String,
}

fn chapter_files() -> Vec<&'static File<'static>> {
    fn walk<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a File<'a>>) {
        for entry in dir.entries() {
            match entry {
                DirEntry::File(file) => out.push(file),
                DirEntry::Dir(subdir) => walk(subdir, out),
            }
        }
    }

    let mut files = Vec::new();
    walk(&BOOK, &mut files);
    const EXCLUDED: &[&str] = &["README.md", "SUMMARY.md"];
    files.retain(|f| {
        f.path().extension().is_some_and(|ext| ext == "md")
            && !EXCLUDED
                .iter()
                .any(|excluded| f.path() == Path::new(excluded))
    });
    files.sort_by_key(|f| f.path().to_path_buf());
    files
}

fn chapter_title(content: &str, fallback_path: &str) -> String {
    content
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .unwrap_or(fallback_path)
        .to_string()
}

fn path_string(file: &File<'_>) -> String {
    file.path().to_string_lossy().replace('\\', "/")
}

/// Full table of contents (path + title), sorted by path.
pub fn chapters() -> Vec<Chapter> {
    chapter_files()
        .into_iter()
        .map(|file| {
            let path = path_string(file);
            let content = file.contents_utf8().unwrap_or_default();
            let title = chapter_title(content, &path);
            Chapter { path, title }
        })
        .collect()
}

/// Raw Markdown content of one chapter, addressed by the `path` reported by
/// `chapters()`/`search()` (e.g. `programming/elements/functions.md`).
pub fn read_chapter(path: &str) -> Option<&'static str> {
    BOOK.get_file(path).and_then(|f| f.contents_utf8())
}

/// Case-insensitive keyword search across chapter paths, titles, and
/// content. Returns up to `limit` matches plus whether results were
/// truncated.
pub fn search(query: &str, limit: usize) -> (Vec<BookMatch>, bool) {
    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();
    let mut truncated = false;

    for file in chapter_files() {
        let path = path_string(file);
        let content = file.contents_utf8().unwrap_or_default();
        let title = chapter_title(content, &path);

        let matching_line = content
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && line.to_lowercase().contains(&query_lower));

        let is_match = matching_line.is_some()
            || path.to_lowercase().contains(&query_lower)
            || title.to_lowercase().contains(&query_lower);
        if !is_match {
            continue;
        }

        if matches.len() >= limit {
            truncated = true;
            break;
        }

        let snippet = matching_line.unwrap_or(&title).to_string();
        matches.push(BookMatch {
            path,
            title,
            snippet,
        });
    }

    (matches, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_known_chapters() {
        let chapters = chapters();
        assert!(chapters.iter().any(|c| c.path == "introduction.md"));
        assert!(chapters
            .iter()
            .any(|c| c.path == "programming/elements/functions.md"));
    }

    #[test]
    fn reads_a_chapter_by_path() {
        let content = read_chapter("introduction.md").expect("introduction.md should exist");
        assert!(!content.is_empty());
    }

    #[test]
    fn searches_by_keyword() {
        let (matches, _) = search("track", 20);
        assert!(!matches.is_empty());
    }
}
