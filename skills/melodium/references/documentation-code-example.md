# Mélodium Documentation — Code Example Strategy

This guide describes when and how to write Mélodium code examples inside documentation comments for library elements (`mel_function`, `mel_treatment`, `mel_data`).

Read this guide alongside the main [Mélodium Documentation](documentation.md) guide, which covers overall doc structure, Mermaid diagrams, and per-element rules.

---

## When to write an example

The Mermaid diagram answers **what** the element does structurally. An example is only warranted when the answer to "how do I actually use this?" is not obvious from the diagram and the parameter/port table alone. Add an example when:

- The element must be **combined with at least one other element** to produce useful output (e.g. `toJson` paired with option unwrapping, or `startup` as a mandatory source).
- The element has a **non-trivial wiring pattern** — multiple inputs that must be satisfied together, split output paths for success/failure, or a model injection.
- The element is a **conversion entry-point** whose output needs to be fed into a domain-specific consumer (e.g. building a JSON object with `fromStringMap` then passing it somewhere).
- The treatment is **generic** and the concrete type constraint is easy to miss without a concrete instantiation.

Do **not** add an example for trivial one-input-one-output stream mappers — the diagram is sufficient.

---

## Structure of an example

An example is a minimal, self-contained Mélodium treatment body (or a full treatment when needed). It should:

1. Show one realistic use case, not an exhaustive API tour.
2. Be **runnable in principle** — every `Self` output must be satisfied, every treatment input must be wired.
3. Use `use` declarations as a preamble when import paths would otherwise be ambiguous.

Template for a full treatment example:

````rust
/// ```
/// use some/area::treatmentA
/// use some/area::treatmentB
///
/// treatment example()
///   input  data:   Stream<string>
///   output result: Stream<string>
/// {
///     treatmentA()
///     treatmentB()
///
///     Self.data -> treatmentA.input,output -> treatmentB.input,result -> Self.result
/// }
/// ```
````

For a simple in-body snippet (when the surrounding treatment is obvious), omit the treatment wrapper and just show the instantiations and connections.

---

## Step-by-step generation process

### Step 1 — Identify the element's role

Determine the element's category from its macro:

| Macro | Role |
|---|---|
| `mel_function` | Pure call used as an argument value (prefix `\|`). Example shows it in a parameter position. |
| `mel_treatment` (Stream in, Stream out) | Pipeline node. Example shows it in a `->` chain between a source and a consumer. |
| `mel_treatment` (Block trigger in) | Event handler. Example always starts from `startup()` or a model source trigger. |
| `mel_treatment` (multiple outputs, some Block) | Success/failure splitter. Example must wire all output branches. |
| `mel_data` | Constructor. Example shows the constructor function(s) and a downstream treatment using the value. |

### Step 2 — Identify required companions

Every treatment needs a **source** for each input. Choose the simplest one:

| Input type | Simplest source |
|---|---|
| `Block<void>` | `startup()` — `startup.trigger` feeds it directly, no extra treatment needed |
| `Block<T>` (data) | `emit<T>(value=…)` triggered by `startup` |
| `Stream<T>` | A realistic upstream — e.g. `readLocal` for bytes, or a prior treatment's output |

Every `Self` output that must be driven needs a **sink**. Prefer:
- `logDebug` / `logDebugs` from `std/engine/log` as a throwaway sink for strings.
- Another treatment that illustrates the natural downstream for the element's output type.

For option-typed outputs, always pair with `unwrap` or `unwrap_or` from `std/ops/option` so the reader sees the complete pattern.

### Step 3 — Write the connection chain

Follow the chaining syntax:

```
source.output -> mid.input,output -> sink.input
```

Rules:
- Use alignment dashes (`-->`, `--->`) only when two parallel wires are shown simultaneously for visual alignment.
- Never introduce a treatment that isn't strictly needed to make the example valid.
- For option outputs, always wire the `none` branch too (e.g. connect both `json` and `error` outputs of `toJson`).

### Step 4 — Choose concrete types for generics

Pick the simplest concrete type that satisfies the constraint:

| Constraint | Default pick |
|---|---|
| `T ()` (unconstrained) | `string` |
| `N: Add + …` (numeric) | `i64` |
| `F: Float` | `f64` |
| `S: ToString` | `string` |
| `B: ToBool` | `bool` |
| `I: ToI64` | `i64` |
| `U: ToU64` | `u64` |

Instantiate with the explicit type argument: `treatmentName<string>()`.

### Step 5 — Verify before writing

- Every input of every instantiated treatment is satisfied by exactly one connection.
- Every `Self` output is driven.
- No `Block` ↔ `Stream` mismatches — check port types in the `#[mel_treatment(...)]` attribute.
- The companion treatments used actually exist in the library — verify import paths.
- If the element has a model parameter, the model is instantiated in the treatment prelude with `model name: ModelType(...)`.

---

## Case-specific patterns

### `mel_function` (pure)

Functions appear as argument expressions, not as wired nodes. Show them in a parameter position:

````rust
/// ```
/// use std/engine/log::logBlock
/// use std/engine/log::|debug
/// use std/engine::Engine
///
/// treatment example()
///   model engine: Engine()
///   input trigger: Block<void>
/// {
///     logBlock[engine=engine](level=|debug(), label="result")
///
///     Self.trigger -> logBlock.message
/// }
/// ```
````

Or as a model parameter default value:

````rust
/// ```
/// use http/server::HttpServer
/// use net/ip::|from_ipv4
/// use net/ip::|localhost_ipv4
///
/// treatment myApp()
///   model server: HttpServer(host=|from_ipv4(|localhost_ipv4()), port=8080)
/// { ... }
/// ```
````

### Success/failure output triples

When a treatment has `completed`/`failed`/`finished` outputs, wire all branches — even if the sink is just a log treatment:

````rust
/// ```
/// use std/engine/log::logDebug
/// use std/engine/log::logError
/// use std/engine::Engine
///
/// treatment example()
///   model engine: Engine()
///   input trigger: Block<void>
/// {
///     executeRaw[sql_pool=pool](sql="DELETE FROM tmp")
///     ok:  logDebug[engine=engine](label="sql")
///     err: logError[engine=engine](label="sql")
///
///     Self.trigger   -> executeRaw.trigger
///     executeRaw.completed -> ok.message
///     executeRaw.failed    -> err.message
/// }
/// ```
````

### Generic stream treatments

Show the generic instantiation explicitly even when it could be inferred, to teach the syntax:

````rust
/// ```
/// fromBool<bool>()
/// Self.values -> fromBool.value,json -> Self.json
/// ```
````

### Option-output treatments

Pair the example with `unwrap` or `unwrap_or` from `std/ops/option` so the reader sees the complete pattern:

````rust
/// ```
/// use std/ops/option::unwrap
///
/// treatment example()
///   input  text: Stream<string>
///   output ip:   Stream<Ipv4>
/// {
///     toIpv4()
///     unwrap<Ipv4>()
///
///     Self.text -> toIpv4.text,ipv4 -> unwrap.option,value -> Self.ip
/// }
/// ```
````

---

## Naming conventions in examples

Apply these consistently — the example is often the reader's first contact with Mélodium syntax:

| Element | Convention | Example |
|---|---|---|
| Treatment calls | camelCase | `fromBool<bool>()`, `toJson()` |
| Function calls | snake_case with `\|` prefix | `\|localhost_ipv4()`, `\|debug()` |
| Model types in prelude | PascalCase | `model db: SqlPool(...)` |
| Context names | `@PascalCase` | `@HttpRequest` |
| Port names | snake_case | `.input`, `.is_json`, `.completed` |
| Parameter names | snake_case | `label="ok"`, `value=42` |
| Aliases | snake_case or descriptive | `ok: logDebug(...)`, `err: logError(...)` |

Treatment names in examples must match Mélodium camelCase call syntax, which corresponds to the name that appears in the `T("...")` label in the Mermaid diagram for that treatment.
