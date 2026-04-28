---
name: melodium
description: Manage Mélodium technology and language. Use when dealing with Mélodium code, if "melodium" is mentionned, or in projects with 'Compo.toml' file, or '.mel' files.
license: EUPL-1.2
compatibility: Requires Mélodium 0.10.0+
allowed-tools: Bash(melodium check *)
---

# Mélodium Language and Technology

Mélodium is a graph-oriented dataflow programming language for distributed orchestration. Programs are defined as explicit graphs of **treatments** (nodes) connected by typed data flows (edges). There is no procedural control flow, no line-by-line execution order: all treatments run as soon as data is available, and all can be considered as running simultaneously.

The reference documentation for the standard library is at **https://doc.melodium.tech/latest/en/**.

---

## Runtime model

When a program launches, Mélodium:
1. Parses and semantically validates all source files.
2. Resolves dependencies.
3. Builds the execution graph.
4. Instantiates models.
5. Triggers tracks and starts execution.

A **track** is the whole ensemble of treatment instances and connections that are created together, live together, and disappear together. Tracks always originate from a model (e.g. one per incoming HTTP connection, one per file found, etc.), and each track is independent — this is the core scalability mechanism.

---

## Elements

### Treatments

The primary computation unit. A treatment declares inputs, outputs, and a body that instantiates other treatments and connects them.

```mel
treatment name<G: TraitA + TraitB>[ModelParam: ModelType](var foo: u64, const bar: string = "default")
  input  in_data:  Stream<G>
  output out_data: Block<void>
{
  // instantiations and connections
}
```

- **Generic parameters** `<G>` — parametrize over a data type; can be constrained with traits.
- **Model configuration parameters** `[Param: Type]` — inject model instances (passed by callers).
- **Treatment parameters** `(...)` — `const` stays the same across all tracks through this instance; `var` can differ per track. Constants can set both const and var parameters; variable elements (parameters, contexts) can only set var parameters.
- **`Self`** — refers to the hosting treatment's own inputs/outputs inside a body.

Order of declarations in a body has no semantic meaning.

```mel
use fs/local::writeLocal
use std/text/convert/string::toUtf8

treatment writeText(var filename: string)
  input  text:          Stream<string>
  output written_bytes: Stream<u128>
{
    writeLocal(path=filename)
    toUtf8()

    Self.text -> toUtf8.text,encoded -> writeLocal.data,amount -> Self.written_bytes
}
```

When the same treatment appears more than once in a body, give it an alias:

```mel
step1: transform<string>()
step2: transform<u64>()
```

### Models

Long-lived stateful resources (database pools, HTTP servers, JS engines, etc.). They live for the entire program execution.

**Declaring a model** (customising a library model type):

```mel
use sql::SqlPool

model MyDatabase(const max: u32 = 5) : SqlPool
{
    min_connections = 1
    max_connections = max
    url = "postgresql://my-user@my-server:4321/my_database"
}
```

**Instantiating a model inside a treatment** (in the treatment prelude, before `{`):

```mel
treatment myApp()
  model database: MyDatabase(max=3)
  input  user_id:   Block<string>
  output user_name: Block<Option<string>>
{
    userData[database=database]()
    Self.user_id -> userData.user_id,user_name -> Self.user_name
}
```

**Receiving a model as a configuration parameter** (so it can be passed down):

```mel
treatment userData[database: SqlPool]()
  input  user_id:  Block<string>
  output user_name: Block<Option<string>>
{
    fetch[sql_pool=database](sql="SELECT name FROM users WHERE id = ?", bindings=["user_id"])
    // ...
}
```

Most of the time, models are instantiated internally and not exposed — users only need to pass required parameters to the call chain.

### Contexts

Data that is available across an entire track, without being explicitly passed between treatments. Contexts are prefixed with `@` and are provided by a source treatment (often a model-linked treatment like `connection` in HTTP).

A treatment that requires a context can **only** be used in a track originating from a source that provides it.

```mel
use http/server::@HttpRequest
use std/data/string_map::|get

treatment actionGetUser()
  require @HttpRequest
  input  data:   Stream<byte>
  output result: Stream<byte>
{
    getUser(user_id = |get(@HttpRequest[parameters], "user"))
    Self.data -> getUser.data,result -> Self.result
}
```

Contexts carry variable data (they come at track creation time).

### Functions

Pure, side-effect-free callables that return a single value. Executed at program initialisation (when used as `const`) or at track creation (when used as `var`). Recognisable by the `|` prefix.

```mel
use net/ip::|localhost_ipv4
use net/ip::|from_ipv4

treatment main()
  model server: HttpServer(
    host=|from_ipv4(|localhost_ipv4()),
    port=8080
  )
{ ... }
```

---

## Inputs, outputs, and connections

### Port types

Every input and output has a type of either `Block<T>` or `Stream<T>`:

| Port kind   | Meaning                                      |
|-------------|----------------------------------------------|
| `Block<T>`  | At most one value — used for discrete events / triggers |
| `Stream<T>` | Continuous sequence — the general case        |

Use `Block<void>` for triggers (something happened) and `Stream<void>` for continuation signals (keep going, no data).

### Connection syntax

Connections are written left-to-right: source output → destination input.

```mel
source.output -> destination.input
```

The arrow can use any number of dashes (`->`  `-->` `--->`…); the dash count has **no semantic meaning** and is only used for visual alignment.

**Chaining:** when a name appears after `treatment_name.` on the **right** of an arrow it is an input; on the **left** it is an output. In a chained form `treatment.input,output`, the first name after `.` is the input and the name after `,` is the output:

```mel
startup.trigger -> emit.trigger,emit -> generate.length,stream -> write.data
```

This reads: `startup.trigger` → `emit.trigger`; `emit.emit` → `generate.length`; `generate.stream` → `write.data`.

**Rules:**
- A connection cannot link different port kinds (`Block` ↔ `Stream` is forbidden).
- A connection cannot create a cycle.
- Each input can only have one incoming connection.
- All inputs of every used treatment must be satisfied.
- All outputs of `Self` (the hosting treatment) must be satisfied.
- Outputs can be connected to any number of inputs, or left unused.

---

## Data types

### Core types

| Unsigned integers | Signed integers | Floating-point | Text     | Logic  |
|-------------------|-----------------|----------------|----------|--------|
| `u8`              | `i8`            | `f32`          | `char`   | `byte` |
| `u16`             | `i16`           | `f64`          | `string` | `bool` |
| `u32`             | `i32`           |                |          | `void` |
| `u64`             | `i64`           |                |          |        |
| `u128`            | `i128`          |                |          |        |

- `byte` — raw 8-bit data, no text assumption.
- `char` — a Unicode scalar value (4 bytes); not equivalent to a byte.
- `string` — UTF-8 text of variable size; not a vector of chars.
- `void` — carries no value, only signals existence.

### Parameterised types

`Vec<T>`, `Option<T>` are built-in. Common library types: `Map<K,V>` (`std/data/map`), `StringMap` (`std/data/string_map`), `Json` (`json`), `HttpStatus` (`http/status`), etc.

### Traits

Traits express intrinsic capabilities of types. Key trait families:

| Family                    | Examples                                      |
|---------------------------|-----------------------------------------------|
| Infallible conversion     | `ToI64`, `ToF64`, `ToString`, `ToByte`, …     |
| Fallible conversion       | `TryToI8`, `TryToU32`, …                      |
| Saturating conversion     | `SaturatingToI64`, …                          |
| Arithmetic (basic)        | `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`, `Pow`, `Euclid` |
| Arithmetic (checked)      | `CheckedAdd`, `CheckedSub`, …                 |
| Arithmetic (saturating)   | `SaturatingAdd`, `SaturatingSub`, `SaturatingMul` |
| Arithmetic (wrapping)     | `WrappingAdd`, `WrappingSub`, `WrappingMul`, `WrappingNeg` |
| Comparison                | `PartialEquality`, `Equality`, `PartialOrder`, `Order` |
| Numeric                   | `Signed`, `Float`, `Bounded`                  |
| Binary ops                | `Binary`                                      |
| Hashing                   | `Hash`                                        |
| Serialisation             | `Serialize`, `Deserialize`                    |
| Display                   | `Display` (human-readable), `ToString` (technical) |

All numeric core types implement the arithmetic and comparison traits; `f32`/`f64` implement `PartialEquality` but not `Equality` (because `NaN ≠ NaN`).

### Generics

Treatments can be generic over types, with optional trait constraints:

```mel
use std/ops/num::isPositive
use std/conv::saturatingToI64

treatment demonstration<N: Float + SaturatingToI64>()
  input  value:      Stream<N>
  output integer:    Stream<i64>
  output is_positive: Stream<bool>
{
    isPositive<N>()
    saturatingToI64<N>()

    Self.value ------> isPositive.value,positive --> Self.is_positive
    Self.value -> saturatingToI64.value,into ------> Self.integer
}
```

---

## Project organisation

### Project tree (development format)

```
my_project/
├── Compo.toml
├── main.mel
├── foo.mel        ← area "foo"
└── foo/
    └── bar.mel    ← area "foo/bar"
```

An **area** corresponds to a `.mel` file. Sub-areas are created by making a folder with the area name and placing `.mel` files inside.

References inside the project use `root` (from the project root) or `local` (relative to current file):

```mel
use root/foo::MyTreatment
use local/bar::OtherTreatment
```

### Compo.toml

```toml
name    = "my_project"
version = "0.1.0"

[dependencies]
std  = "0.10.1"
http = "0.10.1"
fs   = "0.10.1"

[entrypoints]
main   = "my_project/main::main"
server = "my_project/server::serve"
```

- `std` must be declared explicitly even though it is the standard library.
- Version strings follow SemVer compatibility rules (Cargo-style): `"0.10.1"` allows `>=0.10.1, <0.11.0`.
- Other operators: `~0.10.1` (patch only), `0.10.*` (wildcard), `>= 0.9, < 0.11` (range).

### Entrypoints

The `main` entrypoint is called when no command is specified. Other entrypoints are invoked by name:

```shell
melodium myapp.jeu server --port 6789
```

Treatment parameters become CLI arguments automatically. `const` parameters must be known at startup; they become `--flag value` arguments.

### Standalone files

Single-file scripts with a mandatory shebang and metadata header:

```mel
#!/usr/bin/env melodium
#! name    = my_script
#! version = 0.1.0
#! require = std:0.10.1 fs:0.10.1

// Usual Mélodium code…
// Must have exactly one treatment named "main".
```

`require` can be repeated. Standalone files are directly executable when Mélodium is installed.

### Package format (`.jeu`)

Built with `melodium jeu build <project_dir> <output.jeu>`. Already LZMA2-compressed; do not re-compress. Directly executable on any system with Mélodium installed.

---

## CLI reference

| Command                                      | Purpose                                        |
|----------------------------------------------|------------------------------------------------|
| `melodium check <file>`                      | Validate syntax and types (safe, read-only)    |
| `melodium <file>` / `melodium run <file>`    | Run a program (`.mel`, `Compo.toml`, or `.jeu`) |
| `melodium run <file> <cmd> [args…]`          | Run a specific entrypoint                      |
| `melodium info <file>`                       | List entrypoints and options of a program      |
| `melodium new <name>`                        | Scaffold a new package                         |
| `melodium doc --file <file> <output>`        | Generate mdBook documentation                  |
| `melodium jeu build <project> <output.jeu>`  | Bundle a project into a `.jeu` archive         |
| `melodium jeu extract <input.jeu> <output>`  | Extract a `.jeu` archive                       |
| `melodium dist`                              | Start a distribution engine node               |

---

## Standard library packages

| Package      | Provides                                              |
|--------------|-------------------------------------------------------|
| `std`        | Flow control, type conversion, math, logging, engine utilities |
| `fs`         | File-system read/write (`local`, `file`)             |
| `process`    | Subprocess execution                                  |
| `http`       | HTTP client and server                                |
| `net`        | Network primitives (IP addresses, etc.)               |
| `json`       | JSON encode/decode                                    |
| `sql`        | SQL database access                                   |
| `regex`      | Regular expressions                                   |
| `encoding`   | Text encoding/decoding (UTF-8, etc.)                  |
| `javascript` | JavaScript engine integration                         |
| `distrib`    | Distributed-computing coordination                    |
| `work`       | Work queues and resource management                   |
| `cicd`       | CI/CD pipeline utilities                              |

Always check https://doc.melodium.tech/latest/en/ for the exact treatment/function signatures.

---

## How to assist with Mélodium tasks

### Validating code

Run `melodium check <file>` to catch parse and type errors. This tool is already allowed and is always safe to run.

### Writing treatments — step by step

1. Identify data sources: `startup()` trigger, treatment inputs (`Self.*`), or model-initiated tracks.
2. List required transformations — each becomes a treatment instantiation.
3. If the same treatment appears twice, prefix with an alias (`alias: Treatment()`).
4. Wire outputs to inputs using `->` chains, keeping `Block`/`Stream` kinds consistent.
5. Verify every input is satisfied and every `Self` output is driven.

### Use path conventions

Library elements are imported with their full area path:
```mel
use std/flow::emit          // treatment
use std/flow::trigger       // treatment
use std/engine/util::startup
use std/ops/option::|wrap   // function (note | prefix)
use http/server::@HttpRequest  // context (note @ prefix)
```

### Common patterns

**Entry point with startup trigger:**
```mel
use std/engine/util::startup

treatment main() {
    startup()
    // startup.trigger is Block<void> — drives the first step
    startup.trigger -> firstStep.trigger
}
```

**Linear pipeline:**
```mel
treatment pipeline() {
    startup()
    read: readFile(path="/data/input.txt")
    parse: splitLines()
    out:  writeFile(path="/data/output.txt")

    startup.trigger -> read.trigger,data -> parse.data,lines -> out.data
}
```

**Fan-out (same output to multiple inputs):**
```mel
    source.stream --> workerA.input
    source.stream --> workerB.input
```

**Passing a model through the call chain:**
```mel
treatment myApp()
  model db: MyDatabase()
{
    handler[database=db]()
    // ...
}

treatment handler[database: SqlPool]()
  input query: Block<string>
{
    fetch[sql_pool=database](sql="SELECT …", bindings=["query"])
    // ...
}
```

**HTTP server with context:**
```mel
use http/server::HttpServer
use http/server::start
use http/server::connection
use http/method::|get
use http/server::@HttpRequest

treatment myApp()
  model server: HttpServer(host=|from_ipv4(|localhost_ipv4()), port=8080)
{
    startup()
    start[http_server=server]()
    startup.trigger -> start.trigger

    connection[http_server=server](method=|get(), route="/hello")
    handler()

    connection.data -> handler.data,result -> connection.data
}

treatment handler()
  require @HttpRequest
  input  data:   Stream<byte>
  output result: Stream<byte>
{
    // @HttpRequest context is implicitly available here
}
```

### Debugging connection errors

- **Type mismatch** (`Block` ↔ `Stream`): the most common error; verify port kinds match across both ends.
- **Unsatisfied input**: every input of every instantiated treatment must be connected exactly once.
- **Unsatisfied `Self` output**: every output declared on the hosting treatment must be driven.
- **Cycle detected**: rethink the graph topology — Mélodium disallows feedback loops.
- **Context not available**: a treatment with `require @Ctx` can only be used inside a track whose source provides `@Ctx`.
- **Const/var mismatch**: a `var` element (parameter or context) cannot be used to set a `const` parameter.


---

## Further reading

For an extended reference of what is available in Mélodium, see the [Mélodium Reference](references/REFERENCE.md) guide.

If dealing with Mélodium elements written in Rust, read the specific [Mélodium Elements in Rust](references/rust.md) guide.

If dealing with Mélodium documentation, either being instructed to write some or to explore some, read the specific [Mélodium Documentation](references/documentation.md) guide.

### Migrations

If you are asked to migrate code from Github Actions, read the specific [Github Actions migration](references/github-migration.md) guide.

If you are asked to migrate code from Gitlab CI, read the specific [Gitlab CI migration](references/gitlab-migration.md) guide.
