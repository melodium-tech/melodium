# Mélodium MCP server

An [MCP](https://modelcontextprotocol.io/) server exposing Mélodium program
validation and reference lookup to MCP-compatible clients (Claude Desktop,
Claude Code, and similar).

It links the [`melodium`](https://crates.io/crates/melodium) crate directly
and reuses the same loading functions the `melodium check`/`melodium info`
CLI commands call, so behavior stays consistent with the CLI. Every
Mélodium standard library package is compiled in mock mode: no real
filesystem, network, subprocess, or database I/O is performed beyond
reading the program file being validated.

## Usage

Build the server and point an MCP client at the resulting binary over
stdio:

```shell
cargo build --release --package melodium-mcp
```

Add it to your MCP client configuration as a stdio server, using the path
to `target/release/melodium-mcp`.

## Tools

- `check_program` — parse and validate a Mélodium program file (`.mel`,
  `Compo.toml`, or `.jeu`), returning structured errors and the list of
  available entrypoints.
- `get_program_info` — parse a program file and describe each entrypoint:
  identifier, documentation, and parameters (name, const/var, type, default).
- `list_library_elements` — list standard library treatments, functions,
  models, contexts, and data types, optionally filtered by area path (e.g.
  `std/flow`, `http`) and/or kind.
- `describe_element` — full signature of one standard library element by
  identifier (e.g. `std/flow::emit`): documentation, generics, parameters,
  and (for treatments) inputs, outputs, required models and contexts.
- `search_reference` — keyword search across standard library identifiers
  and documentation.
- `get_language_guide` — explanation of Mélodium's dataflow execution model
  (treatments, tracks, models, contexts, connections, generics, project
  layout), aimed at an AI reading or writing Mélodium code.
- `get_cicd_migration_guide` — reference guide for migrating a GitHub
  Actions or GitLab CI pipeline to Mélodium's `cicd` package.
- `search_book` — keyword search across the Mélodium book's chapters
  (titles and content); omit the query to list every chapter.
- `read_book_chapter` — full Markdown content of one book chapter, by the
  path reported by `search_book`.

`list_library_elements`, `describe_element`, and `search_reference` operate
on the full standard library collection, loaded once at startup and reused
across calls. `get_language_guide` and `get_cicd_migration_guide` serve the
same reference documents shipped as the `melodium` Claude Code skill
(`skills/melodium/`), and `search_book`/`read_book_chapter` serve a vendored
copy of the [Mélodium book](https://gitlab.com/melodium/book) (see
`melodium-mcp/book/README.md`) — both embedded into the binary at compile
time, so no network access is required at runtime.
