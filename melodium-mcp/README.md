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

`list_library_elements`, `describe_element`, and `search_reference` operate
on the full standard library collection, loaded once at startup and reused
across calls.
