
# Changelog

## [v0.10.4] (2026-09-24)

- Fixing the crate build: the AI-facing guides (language/runtime model, CI/CD migration references) were embedded via `include_str!` pointing outside the crate directory (`../../skills/melodium/...`), which isn't included when the crate is packaged/published. They are now vendored under `melodium-mcp/guides/`.

## [v0.10.3] (2026-09-08)

- Bundling the Mélodium book and AI-facing guides (language/runtime model, GitHub/GitLab CI/CD migration references) into the server, so any MCP client can retrieve them.
- Fixing the `cargo` package keyword.

## [v0.10.2] (2026-08-04)

First release.
