use melodium::LoadingConfig;
use melodium_common::descriptor::{Collection, Entry, Identified};
use std::sync::Arc;

/// Loads the full core library collection (every compiled-in Mélodium standard
/// library package, in mock mode) once. Meant to be cached for the lifetime of
/// the server, since rebuilding it per request would be wasteful and it never
/// changes at runtime.
pub fn load_core_collection() -> Arc<Collection> {
    let (_, collection) = melodium::load_all(LoadingConfig::new())
        .into_result()
        .expect("failed to load Mélodium core library packages");
    collection
}

pub fn entry_kind(entry: &Entry) -> &'static str {
    match entry {
        Entry::Context(_) => "context",
        Entry::Data(_) => "data",
        Entry::Function(_) => "function",
        Entry::Model(_) => "model",
        Entry::Treatment(_) => "treatment",
    }
}

pub fn entry_documentation(entry: &Entry) -> &str {
    match entry {
        Entry::Context(c) => c.documentation(),
        Entry::Data(d) => d.documentation(),
        Entry::Function(f) => f.documentation(),
        Entry::Model(m) => m.documentation(),
        Entry::Treatment(t) => t.documentation(),
    }
}

pub fn entry_identifier_string(entry: &Entry) -> String {
    entry.identifier().to_string()
}

/// First non-empty line of a documentation string, for compact listings.
pub fn summary_line(documentation: &str) -> String {
    documentation
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .to_string()
}
