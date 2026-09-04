# 01: Hello, Mélodium

**Concepts introduced:** treatments & connections, `Block<T>` vs `Stream<T>`, functions (`|name(...)`), `startup()`.

The smallest useful Mélodium program. It has no procedural entry point: `main` is a *graph* of treatments wired together, not a function body that runs top to bottom. `startup()` fires once when the engine starts, and every other treatment reacts as soon as the data it needs is available.

## What it does

- Computes a single greeting with a pure function and writes it to a file (the `Block` half).
- Generates the same name repeated `times` as a stream and numbers it (the `Stream` half).
- Logs both, so a run shows the two execution styles side by side.

```
melodium run Compo.toml --name "World" --times 3
```

## How it is built

No models here: this example only uses stateless treatments and functions.

### Data flow

```
                              ┌── emit(greeting) ── stream() ── writeTextLocal ── logInfoMessage
startup.trigger ── fan-out ──┤
                              └── emit(times) ── generate ──┬── logInfos (each greeting)
                                                             └── count ── toString ── logInfos (running count)
```

## Runtime behaviour

1. `startup.trigger` fires once and fans out to both halves of the graph: nothing here runs "first" or "second", both start as soon as the trigger arrives.
2. **Block side:** `|format` and `|map`/`|entry` build the greeting string from the `name` parameter, purely (no I/O); `emit` turns it into a `Block<string>`; `stream` widens it into a one-element `Stream<string>` so `writeTextLocal` can consume it; the file write logs a confirmation once finished.
3. **Stream side:** `emit<u128>(value=times)` supplies the length to `generate`, which repeats `name` that many times as a `Stream<string>`. That stream fans out to `logInfos` directly, and separately through `count` and `toString` before being logged again. `count` numbers elements starting at **0**, not at 1 as its own documentation claims: a run of `--times 3` logs `0`, `1`, `2`. Example 03 shows how to correct for that when a total matters.

### Key Mélodium patterns used

- **Functions vs treatments**: `|format`, `|map`, `|entry` are pure functions (no ports, called inline as values); `emit`, `stream`, `generate`, `count` are treatments (they have ports and live in the graph).
- **Function positional arguments**: Mélodium functions take positional arguments in their *declared* order, e.g. `|format(format_string, entries)`. This does not always match alphabetical order, so when in doubt check a working example or validate with `melodium check`.
- **Fan-out**: `startup.trigger` drives two independent branches of the graph from a single output.
- **`Block<T>` → `Stream<T>`**: `std/flow::stream` is the standard way to turn a one-shot value into a one-element stream when a downstream treatment expects a `Stream`.

Next: [02_flow_and_generics](../02_flow_and_generics/) introduces generics, traits, and stream filtering.
