# Mélodium Projects — Creation and Iteration Guide

This guide covers everything needed to create a Mélodium project from scratch: file layout, `Compo.toml` authoring, entrypoints, `.mel` file structure, and the check-run-fix iteration loop you must follow after writing code.

---

## 1. Scaffolding a new project

### With the CLI

```shell
melodium new my_project
```

This creates:

```
my_project/
├── Compo.toml
└── lib-root.mel
```

`lib-root.mel` is empty. Edit `Compo.toml` to add dependencies before writing any code that imports from packages.

### By hand

Create the directory, write `Compo.toml`, then create `.mel` files. There is no build step — Mélodium parses source files directly.

---

## 2. Compo.toml

`Compo.toml` is the project manifest. It must be at the project root.

```toml
name    = "my_project"
version = "0.1.0"

[dependencies]
std  = "0.10.1"
http = "0.10.1"
fs   = "0.10.1"
json = "0.10.1"

[entrypoints]
main   = "my_project/main::main"
server = "my_project/server::serve"
```

### Rules

- `name` becomes the **namespace prefix** used in import paths and entrypoint references. Use only alphanumeric characters and underscores.
- `version` follows SemVer. It is informational for development projects and normative for published packages.
- `std` must be declared explicitly even though it is the standard library. It is never implicit.
- **All packages you import must appear in `[dependencies]`** — a missing dependency causes a validation error.

### Version specifiers

| Syntax | Meaning |
|--------|---------|
| `"0.10.1"` | `>=0.10.1, <0.11.0` (minor-compatible) |
| `"~0.10.1"` | `>=0.10.1, <0.10.2` (patch-only) |
| `"0.10.*"` | Any patch of `0.10` |
| `">= 0.9, < 0.11"` | Explicit range |

For active Mélodium development, prefer `"0.10.1"` (minor-compatible). All standard-library packages share the same version number as the engine.

### Common dependency set

For most projects start with:

```toml
[dependencies]
std  = "0.10.1"   # always required
fs   = "0.10.1"   # file I/O
http = "0.10.1"   # HTTP client/server
json = "0.10.1"   # JSON
```

Add `process`, `sql`, `regex`, `net`, `javascript`, `distrib`, `work`, `cicd` only when needed.

---

## 3. File and area layout

Every `.mel` file defines an **area**. The area name is derived from the file's path relative to the project root.

```
my_project/
├── Compo.toml
├── main.mel          → area  "my_project/main"
├── util.mel          → area  "my_project/util"
└── http/
    ├── server.mel    → area  "my_project/http/server"
    └── client.mel    → area  "my_project/http/client"
```

The project name from `Compo.toml` is the root namespace. A file at `main.mel` belongs to `my_project/main`.

### Importing within the project

```mel
use root/http/server::handleRequest   // from any file: absolute from project root
use local/util::formatPath            // relative to the current file's area
```

Prefer `root/…` imports for clarity; `local/…` is convenient for closely related files in the same subdirectory.

---

## 4. The main.mel entry-point treatment

Every project needs at least one entry-point treatment. The minimal pattern using `startup`:

```mel
use std/engine/util::startup

treatment main() {
    startup()
    startup.trigger -> firstStep.trigger
}
```

`startup` fires exactly once when the engine is initialised. Its single output `trigger: Block<void>` is the universal first-step driver.

### Treatment with CLI parameters

Treatment parameters become CLI flags automatically. `const` parameters must be known at startup (they become `--flag value` arguments on the command line):

```mel
treatment main(const host: string = "127.0.0.1", const port: u16 = 8080) {
    startup()
    // …
}
```

Run with: `melodium Compo.toml -- --host 0.0.0.0 --port 9000`

`var` parameters can be set from both `const` sources and `var` sources; `const` parameters can only be set from `const` sources.

---

## 5. Entrypoints in Compo.toml

```toml
[entrypoints]
main   = "my_project/main::main"
worker = "my_project/worker::run"
```

- The key (`main`, `worker`) is the command name.
- The value is `<namespace>/<area>::<treatment>`.
- `main` is special: it runs when no command is given (`melodium Compo.toml`).
- Other entrypoints are invoked by name: `melodium Compo.toml worker --threads 4`.

The referenced treatment must exist in the corresponding `.mel` file, and must have no required inputs (entrypoints take no data inputs).

---

## 6. Structuring multi-file projects

Split code across files when:
- A treatment body exceeds ~60 lines of wiring.
- A logical sub-domain (HTTP handling, database, file processing) can be isolated.
- You want to reuse a set of treatments from multiple entry points.

Typical layout for a web application:

```
my_app/
├── Compo.toml
├── main.mel           ← startup, model declarations, top-level wiring
├── routes/
│   ├── users.mel      ← user-related HTTP handlers
│   └── health.mel     ← health-check route
└── db/
    └── queries.mel    ← SQL fetch/execute treatments
```

`main.mel` instantiates models and passes them down via model configuration parameters `[model: ModelType]`.

---

## 7. Model declaration pattern

Models live for the entire program. Declare them in the top-level treatment and pass them down.

```mel
// main.mel
use sql::SqlPool
use http/server::HttpServer
use net/ip::|localhostIpv4
use net/ip::|fromIpv4
use root/routes/users::handleUsers

model AppDatabase(const url: string) : SqlPool {
    min_connections = 1
    max_connections = 10
}

treatment main(const db_url: string = "postgresql://localhost/myapp")
  model db: AppDatabase(url=db_url)
  model server: HttpServer(host=|fromIpv4(|localhostIpv4()), port=8080)
{
    startup()
    start[http_server=server]()
    startup.trigger -> start.trigger

    handleUsers[database=db, http_server=server]()
}
```

```mel
// routes/users.mel
use sql::SqlPool
use http/server::HttpServer
use http/server::connection
use http/method::|get

treatment handleUsers[database: SqlPool, http_server: HttpServer]() {
    connection[http_server=http_server](method=|get(), route="/users")
    // …
}
```

---

## 8. Common treatment body structure

When writing a treatment body, follow this order:

1. **Declare aliases** for any treatment type used more than once.
2. **Instantiate treatments** (and inline model instantiations if needed).
3. **Write connections**, left-to-right, grouping related chains on the same line.

```mel
use std/engine/util::startup
use std/flow::emit
use std/text/convert/string::fromUtf8
use std/text/compose::split
use std/flow/vec::flatten

treatment readLines(const path: string, const filesystem: FileSystem)
  input  trigger:    Block<void>
  output lines:      Stream<string>
  output completed:  Block<void>
  output failed:     Block<void>
{
    readFile: read()
    toText:   fromUtf8()
    toLines:  split(delimiter="\n", inclusive=false)
    toStream: flatten<string>()

    pathEmit: emit<string>(value=path)
    fsEmit:   emit<FileSystem>(value=filesystem)

    Self.trigger -> pathEmit.trigger,emit -> readFile.path
    Self.trigger -> fsEmit.trigger,emit  -> readFile.filesystem
    Self.trigger -> readFile.trigger

    readFile.data      -> toText.encoded,text    -> toLines.text,splitted -> toStream.vector,value -> Self.lines
    readFile.completed -> Self.completed
    readFile.failed    -> Self.failed
}
```

Ensure:
- Every input of every instantiated treatment is connected exactly once.
- Every `Self` output is driven.
- No `Block` ↔ `Stream` mismatches across connections.

---

## 9. Iteration loop: check → fix → check → run

**Always run `melodium check` after writing or editing any `.mel` file.** This is the primary feedback loop. It is fast, read-only, and catches all parse and type errors before running.

### Check a single file

```shell
melodium check main.mel
```

### Check the whole project

```shell
melodium check Compo.toml
```

### Read the error output

`melodium check` prints structured errors with file, line, and column. Common categories and what they mean:

| Error message pattern | Likely cause | Fix |
|-----------------------|-------------|-----|
| `undeclared element` / `unknown use` | Import path wrong or package not in `[dependencies]` | Verify import path; add package to `Compo.toml` |
| `input … not connected` | A treatment input was never wired | Add a `->` connection to that input |
| `output … not satisfied` | A `Self` output is never driven | Wire a treatment output to `Self.<output>` |
| `Block`/`Stream` mismatch | Connected ports have incompatible kinds | Check both sides; convert with `stream<T>` or `trigger<T>` |
| `cannot use var … for const` | A `var` source (context, var parameter) feeds a `const` parameter | Use only `const` sources for `const` parameters |
| `context … not available` | `require @Ctx` used outside a track that provides it | Only call such treatments inside tracks from the providing model |
| `cycle detected` | Connection graph has a loop | Rethink the topology; Mélodium disallows feedback |
| `alias required` | Same treatment type appears twice without aliasing | Prefix instances: `a: MyTreatment()` / `b: MyTreatment()` |

### After check passes, run

```shell
melodium run Compo.toml
melodium run Compo.toml worker --threads 4   # named entrypoint with args
```

Or run a standalone `.mel` file:

```shell
melodium run main.mel
```

### Other useful CLI commands

| Command | Purpose |
|---------|---------|
| `melodium check Compo.toml` | Validate the full project (use this by default) |
| `melodium info Compo.toml` | List entrypoints and their parameters |
| `melodium jeu build . output.jeu` | Bundle project into a portable archive |
| `melodium doc --file Compo.toml ./doc-out` | Generate mdBook documentation |

---

## 10. Iterating on errors — practical checklist

After running `melodium check`, for each error:

1. **Read the file and line indicated.** The column points to the first problematic token.
2. **Identify the element kind** (treatment, model, function, context) — the error message states it.
3. **Check imports first.** A missing `use` or a typo in the area path causes cascading undeclared-element errors. Fix imports before chasing downstream errors.
4. **Verify port kinds.** If you see a Block/Stream mismatch, look at the signatures of both the source output and destination input; use `trigger<T>` to get a `Block<void>` from a stream's start, or `stream<T>` to lift a Block to a Stream.
5. **Run check again immediately** after each fix. Do not batch fixes — errors can mask each other.
6. **Repeat until zero errors**, then run the project.

---

## 11. Standalone script format (alternative to Compo.toml)

For single-file scripts that do not need a project directory:

```mel
#!/usr/bin/env melodium
#! name    = my_script
#! version = 0.1.0
#! require = std:0.10.1 fs:0.10.1

use std/engine/util::startup
use fs/file::read

treatment main() {
    startup()
    // …
}
```

- The shebang and `#!` metadata lines are mandatory.
- `require` lists packages separated by spaces, each as `name:version`.
- The file must contain exactly one treatment named `main`.
- Check: `melodium check my_script.mel`
- Run: `melodium run my_script.mel` or `./my_script.mel` (if executable bit is set).
