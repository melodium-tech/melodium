# Vendored AI-facing guides

This directory is a vendored copy of `skills/melodium/` (the `SKILL.md`
language/runtime guide and its `references/github-migration.md` /
`references/gitlab-migration.md` CI/CD migration references) from this same
repository. It is embedded into the `melodium-mcp` binary at compile time via
`include_str!`.

The copy exists because `cargo package` only includes files below the crate
root (`melodium-mcp/`); the crate cannot reference `../../skills/melodium/`
by relative path once published, which broke installing `melodium-mcp` from
crates.io. See `melodium-mcp/book/README.md` for the same rationale applied
to the vendored Mélodium book.

This content may fall behind `skills/melodium/` over time and needs to be
refreshed manually from there occasionally (re-copy `SKILL.md` and
`references/*.md` over this directory, keeping this README).
