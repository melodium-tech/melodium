# Mélodium Documentation Rules

This guide covers how to write documentation for Mélodium elements — both in Rust source files (using `mel_*` macros) and in `.mel` source files.

The `melodium doc` command generates an mdBook (Markdown-based) site from the documentation strings attached to each element. Whatever you write in the doc comment appears verbatim in the generated HTML page for that element, rendered as Markdown. Mermaid diagrams embedded as fenced code blocks with the ` ```mermaid ` language tag are rendered as interactive graphs.

---

## Where documentation is written

### In Rust files

Use standard `///` line doc comments placed **above the `#[mel_*]` attribute**, not above the `fn` or `struct` keyword:

```rust
/// Short one-line summary.
///
/// Longer explanation if needed.
#[mel_treatment( ... )]
pub async fn my_treatment() { ... }
```

The doc comment must come first; the macro attribute immediately follows. Never put the doc comment between the macro attribute and the function definition.

### In `.mel` files

Use block comments delimited by `/** ... */` placed **immediately before the treatment, model, or function declaration**:

```mel
/**
Short one-line summary.

Longer explanation if needed.
*/
treatment myTreatment(param: string)
  input data: Stream<byte>
  output result: Stream<byte>
{ ... }
```

Single-line `//` comments and `/* ... */` block comments also work syntactically but are not forwarded to the documentation system. Only `/** ... */` produces documented entries.

---

## Structure of a documentation comment

### First line — the summary

The first line is the short summary that appears in index listings and search results. Keep it to one sentence, ending without a period. It answers "what does this element do?" in the most concise possible way.

```rust
/// Merge two streams.
```

```mel
/**
Merge two streams.
*/
```

### Body — the explanation

After a blank line, add as much detail as needed. The body is plain Markdown. Cover:

1. **What the element does** — the effect, not the implementation.
2. **Port semantics** — describe each non-obvious input and output: when it fires, what it contains, whether it can be absent.
3. **Parameter semantics** — describe parameters whose meaning isn't obvious from the name.
4. **Edge cases and invariants** — what happens when a stream ends early, when a block never arrives, when data is invalid, etc.

Reference ports and parameters with backtick-quoted names: `` `data` ``, `` `trigger` ``, `` `path` ``.

### Notes and warnings

Use emoji prefixes for inline callouts; this is the established convention across the codebase:

| Prefix | Meaning |
|--------|---------|
| `ℹ️` | Informational note — non-obvious behaviour the reader should know |
| `⚠️` | Warning — pitfall, misuse risk, or sharp edge |

```rust
/// ℹ️ `start` and `first` are always emitted together.
/// If the stream only contains one element, `first` and `last` both contain it.
///
/// ⚠️ Use `HttpServer` with `connection` treatment, as using `incoming` source
/// and `outgoing` treatment directly should be done carefully.
```

Do not use headers (`##`, `###`) inside a single element's doc comment. The generated page already has section headers (`#### Inputs`, `#### Outputs`, etc.) inserted by the doc renderer above the doc string. Use flat paragraphs and lists instead.

---

## Documenting ports and parameters in the body

The doc renderer automatically generates the `#### Inputs`, `#### Outputs`, `#### Parameters` sections from the element's descriptor. Do not duplicate those tables inside the doc text.

Instead, describe the **semantics** of ports in prose, referencing them by name:

```rust
/// The content of the file given through `path` is streamed through `data`.
/// When the file is reached and opened, `reached` is emitted.
/// Once the file is totally and successfully read, `completed` is emitted.
/// `finished` is emitted when the read ends, regardless of the reason.
/// All reading errors are streamed through `errors`.
///
/// If any reading failure happens, `failed` is emitted.
```

For models, describe each parameter's purpose in a bullet list if there are several:

```rust
/// - `host`: the network address to bind with.
/// - `port`: the port to bind with.
```

For contexts, use the same bullet-list style for fields:

```rust
/// - `id`: identifier of connection, uniquely identifies an HTTP connection.
/// - `route`: the route matched by the request.
/// - `path`: the actual path called by the request.
/// - `parameters`: the parameters from the route.
/// - `method`: the HTTP method used by the request.
```

---

## Mermaid diagrams

**Every treatment that has at least one input and one output should include a Mermaid graph.** Diagrams are the primary communication tool for data flow — they make the port structure and the direction of data immediately obvious to the reader.

Embed the diagram as a fenced code block with the `mermaid` language tag:

````rust
/// ```mermaid
/// graph LR
///     T("emit(value=🟨)")
///     B["〈🟦〉"] -->|trigger| T
///     T -->|emit| S["〈🟨〉"]
///
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
/// ```
````

### Graph layout conventions

Always use `graph LR` (left-to-right). Inputs enter from the left, the treatment node is in the centre, outputs exit to the right.

```
graph LR
    T("treatmentName()")
    InputA["..."] -->|input_port| T
    T -->|output_port| OutputB["..."]
```

### Treatment node label

Label the treatment node using its Mélodium call syntax, including any key parameters when they affect the visual:

```
T("emit(value=🟨)")
T("split(delimiter=…)")
T("generate()")
```

### Data representation with emoji

Use coloured squares and circles as visual tokens for data flowing through ports. There is no strict assignment — pick colours that are visually distinct and consistent within a single diagram:

| Visual | Typical use |
|--------|-------------|
| `🟦 🟩 🟨 🟧 🟥 🟪 🟫` | Stream elements (coloured squares = values in a stream) |
| `〈🟦〉` | Block value (angle brackets around a square = single block event) |
| `…` | Ellipsis indicating that the stream continues |
| `［🟦 🟦 🟦］` | Vec (square brackets = a vector of values) |

Example for a stream input and a Block output:

```
B["🟥 … 🟨 🟨 🟨 … 🟩"] -->|stream| T
T -->|start| S["〈🟦〉"]
T -->|end|   E["〈🟦〉"]
```

Example for two inputs merged non-deterministically:

```
A["… 🟦 🟫 …"] -->|a| T
B["… 🟧 🟪 🟨 …"] -->|b| T
T -->|value| V["… 🟦 🟧 🟪 🟫 🟨 …"]
```

### Removing node borders

All input and output data nodes should have their borders removed so they don't draw attention away from the treatment node and the port labels:

```
style A fill:#ffff,stroke:#ffff
style B fill:#ffff,stroke:#ffff
style O fill:#ffff,stroke:#ffff
```

Apply this `style` line for every non-treatment node in the diagram.

### Full diagram example — `chain`

````rust
/// ```mermaid
/// graph LR
///     T("chain()")
///     A["🟨 🟨 🟨 🟨 🟨 🟨"] -->|first| T
///     B["… 🟪 🟪 🟪"] -->|second| T
///     T -->|chained| O["… 🟪 🟪 🟪 🟨 🟨 🟨 🟨 🟨 🟨"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
````

### Full diagram example — `trigger`

````rust
/// ```mermaid
/// graph LR
///     T("trigger()")
///     B["🟥 … 🟨 🟨 🟨 🟨 🟨 🟨 … 🟩"] -->|stream| T
///
///     T -->|start| S["〈🟦〉"]
///     T -->|first| F["〈🟩〉"]
///     T -->|last| L["〈🟥〉"]
///     T -->|end| E["〈🟦〉"]
///
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style L fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
/// ```
````

---

## Mélodium code examples

When the usage of an element is non-obvious, include a Mélodium code example. Use a plain fenced code block with no language tag, or the `mel` language tag if the renderer supports it. Code blocks in doc comments render as `<pre>` blocks in the generated HTML.

Examples are especially useful for:

- Treatments that need to be combined with others to be useful
- Treatments with non-trivial parameter combinations
- Showing the difference between related functions (e.g. a streaming treatment and its function counterpart)

### Example for a text treatment

````rust
/// Rescale stream of strings.
///
/// Unscaled stream can be cut at any position:
/// ```
/// "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean qua"
/// "m velit, tristique et arcu in, viverra pulvinar ante."
/// ```
///
/// While treatments may expect well-defined strings:
/// ```
/// "Lorem ipsum dolor sit amet, consectetur adipiscing elit."
/// "Aenean quam velit, tristique et arcu in, viverra pulvinar ante."
/// ```
````

### Example for a treatment in context

In `.mel` files, include a full usage snippet showing the treatment connected in a graph:

```mel
/**
Connection on HTTP server.

For every valid incoming HTTP request to `http_server`, a new track is created.

In order to start sending a response, `status` and `headers` inputs must be filled.

Outputs:
    - `headers`: headers sent in request.
    - `data`: the raw body data received in the request.
    - `failed`: emitted if a failure occurs.
    - `error`: error message if a failure occurs.

Inputs:
    - `status`: HTTP status response.
    - `headers`: headers to send in response.
    - `data`: data to send in response (the HTTP body).
*/
treatment connection[http_server: HttpServer](const method: HttpMethod, const route: string)
  ...
```

---

## What to document per element kind

### Treatments

- One-line summary.
- Describe the flow: what triggers the treatment, what data flows through each port, what causes completion.
- Describe all non-obvious outputs (especially `finished`/`completed`/`failed` triples — always explain which fires when).
- Include a Mermaid diagram.
- Include a code example if the wiring is non-trivial.

### Functions

- One-line summary.
- Describe any non-obvious return value (especially for fallible functions returning `Option`).
- State when `None` is returned.
- No diagram needed for functions.

### Models

- One-line summary.
- Describe each parameter using a bullet list.
- Describe the sources and what creates a track.
- Note if the model should be used via a specific companion treatment (e.g. `HttpServer` → `connection`).

### Data types

- One-line summary saying what the type represents.
- If any trait implementations have non-obvious behaviour, call them out explicitly (e.g. `ToString` vs `TryToString` on `Json`).

### Contexts

- One-line summary.
- Bullet list of each field with its meaning.

---

## Generated page structure

Understanding what the renderer generates helps write docs that complement rather than repeat the automatic content.

### Unicode symbols used in generated pages

The renderer uses specific Unicode glyphs as visual prefixes in all auto-generated sections. Knowing them helps you understand the rendered output and write prose that fits alongside it:

| Symbol | Meaning | Where it appears |
|--------|---------|-----------------|
| `⤇` | Treatment | SUMMARY.md list entries, Sources section of models |
| `⬢` | Model | SUMMARY.md list entries |
| `◼` | Data type | SUMMARY.md list entries |
| `𝑓` | Function | SUMMARY.md list entries |
| `⥱` | Context | SUMMARY.md list entries, Required/Provided contexts sections |
| `◻` | Generic type parameter | Generics section |
| `↳` | Parameter | Parameters section |
| `⇥` | Input port | Inputs section |
| `↦` | Output port | Outputs section |
| `↴` | Return value | Return section of functions |
| `↪` | Context field entry | Entries section of contexts |
| `∈` | Trait membership | Traits section of data types |
| `⬡` | Model configuration | Configuration section of treatments |

These symbols appear in the auto-generated sections, not in your doc string. Do not use them in your documentation prose — they are reserved for the renderer.

### Page templates

For a **treatment**, the generated page is:

```
# Treatment <name>

`<fully-qualified-id>`

---

#### Generics
◻ `T` _(any)_
◻ `N:` `Trait1` + `Trait2`

#### Configuration
⬡ `model_name:` [`pkg/area::ModelType`](link)

#### Provide contexts
⥱ `@CtxName:` [`pkg::@CtxName`](link) from `model_name:` [`pkg::ModelType`](link)

#### Parameters
↳ `var param_name:` `Type = default`
↳ `const param_name:` `Type`

#### Required contexts
⥱ `@CtxName:` [`pkg::@CtxName`](link)

#### Inputs
⇥ `port_name:` `Block<Type>` _([`pkg::Type`](link))_
⇥ `port_name:` `Stream<Type>`

#### Outputs
↦ `port_name:` `Block<void>`
↦ `port_name:` `Stream<Type>` _([`pkg::Type`](link))_

---

<doc string rendered as Markdown>   ← your documentation
```

For a **function**:

```
# Function |name

`pkg/area::|name`

---

#### Usage
```
|name<T>(param1, param2)
```

#### Generics
◻ `T:` `Trait`

#### Parameters
↳ `param_name:` `Type`

#### Return
↴ `ReturnType`

---

<doc string>
```

For a **model**:

```
# Model <name>

`pkg/area::ModelName`

Based on [`pkg::ParentModel`](link)    ← only if applicable

---

#### Parameters
↳ `const param_name:` `Type = default`
↳ `const required_param:` `Type`


---

#### Sources
⤇ `sourceName:` [`pkg/area::sourceName`](link)
⤇ `sourceName:` [`pkg/area::sourceName`](link) with [`@CtxName`](link)


---

<doc string>
```

Note: `Parameters` and `Sources` are each separated by their own `---` divider. Sources that create tracks with a context show `with [`@CtxName`](link)` after the treatment link. All model parameters are shown as `const`. Source entries link to a companion treatment page (e.g. `sql::connected`, `sql::failure`) that documents the outputs that source creates.

For a **data type**:

```
# Data <name>

`pkg/area::TypeName`

---

#### Traits
∈ `TraitName`
∈ `OtherTrait`

---

<doc string>
```

For a **context**:

```
# Context @<name>

`pkg/area::@ContextName`

---

#### Entries
↪ `field_name:` `Type` _([`pkg::Type`](link))_

---

<doc string>
```

The doc string is always rendered **after** the auto-generated sections. Write your documentation to extend, not restate, what the renderer already shows.

### Index page structure

Area index pages list entries grouped by kind, each prefixed with its Unicode symbol. The exact spacing between symbol and name differs by element type — this is how the renderer generates it:

```
# Area <area-name>

`pkg/area`

---

## Subareas

[subarea](subarea/index.md)

## Data types

◼[ TypeName](TypeName.md)

## Contexts

⥱ [@ContextName](@ContextName.md)

## Functions

𝑓 [|function_name](|function_name.md)

## Models

⬢[ ModelName](ModelName.md)

## Treatments

⤇[ treatmentName](treatmentName.md)
```

Spacing rules in index pages (generated by the renderer, not your concern when writing docs):

| Element type | Spacing pattern | Example |
|-------------|----------------|---------|
| Treatment | `⤇[ name](...)` | `⤇[ emit](emit.md)` |
| Model | `⬢[ name](...)` | `⬢[ HttpServer](HttpServer.md)` |
| Data type | `◼[ name](...)` | `◼[ Level](Level.md)` |
| Context | `⥱ [@name](...)` | `⥱ [@HttpRequest](@HttpRequest.md)` |
| Function | `𝑓 [|name](...)` | `𝑓 [|to_string](\|to_string.md)` |

Note in SUMMARY.md the patterns are slightly different (space after symbol, no bracket-space):

| Element type | SUMMARY pattern |
|-------------|----------------|
| Treatment | `[⤇ name](path)` |
| Model | `[⬢ name](path)` |
| Data type | `[◼ name](path)` |
| Context | `[⥱ @name](path)` |
| Function | `[𝑓 \|name](path)` |

Function filenames and identifiers always retain the `|` prefix literally: the file for `|to_string` is named `|to_string.md` and the fully-qualified id is `std/conv::|to_string`.

---

## Summary checklist

When writing documentation for a Mélodium element, verify:

- [ ] First line is a short, clear summary (no trailing period).
- [ ] Non-obvious ports are explained by name in prose.
- [ ] `finished`/`completed`/`failed` outputs are explained if present.
- [ ] Notes and warnings use `ℹ️` / `⚠️` prefixes.
- [ ] A Mermaid `graph LR` diagram is included for treatments with inputs and outputs.
- [ ] All non-treatment nodes in the diagram have `style X fill:#ffff,stroke:#ffff`.
- [ ] Data tokens in the diagram use emoji squares/circles consistent within the diagram.
- [ ] A code example is included when wiring is non-trivial or usage is not obvious.
- [ ] No section headers (`##`, `###`) inside the doc comment body.
- [ ] No restating of information the renderer already generates (parameter names/types, input/output types).

For detailed guidance on writing and validating code examples specifically, see the [Code Example Strategy](documentation-code-example.md) reference.
