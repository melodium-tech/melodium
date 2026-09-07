# 08: JavaScript Transform

**Concepts introduced:** the `JavaScriptEngine` model, transforming structured data with JS.

**Book:** [Models](https://doc.melodium.tech/book/en/programming/elements/models.html) (a third flavor of the same idea: a long-lived model holding compiled code, rather than a network or database connection).

Reads one JSON object per line, computes a letter grade from its `score` field in JavaScript, and writes the graded records back out.

## What it does

```
melodium run Compo.toml --input_file students.json
```

```json
{"grade":"A","name":"Ada","score":92}
{"grade":"D","name":"Grace","score":58}
{"grade":"C","name":"Linus","score":74}
{"grade":"F","name":"Barbara","score":45}
```

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `engine` | `JavaScriptEngine` | Loads the `grade()` function once at startup; reused for every line |

### Data flow

```
readTextLocal ──▶ lines ──▶ toJson ──▶ process (JS grade()) ──▶ toString ──▶ log + writeTextLocal
```

### Reference

- **`fs`**: [readTextLocal](https://doc.melodium.tech/latest/en/fs/local/readTextLocal.html), [writeTextLocal](https://doc.melodium.tech/latest/en/fs/local/writeTextLocal.html)
- **`javascript`**: [JavaScriptEngine](https://doc.melodium.tech/latest/en/javascript/JavaScriptEngine.html), [process](https://doc.melodium.tech/latest/en/javascript/process.html)
- **`json`**: [Json](https://doc.melodium.tech/latest/en/json/Json.html), [toJson](https://doc.melodium.tech/latest/en/json/toJson.html), [|null](https://doc.melodium.tech/latest/en/json/value/|null.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logInfos](https://doc.melodium.tech/latest/en/std/engine/log/logInfos.html), [filter](https://doc.melodium.tech/latest/en/std/flow/filter.html), [flatten](https://doc.melodium.tech/latest/en/std/flow/vec/flatten.html), [split](https://doc.melodium.tech/latest/en/std/text/compose/split.html), [trim](https://doc.melodium.tech/latest/en/std/text/compose/trim.html), [format](https://doc.melodium.tech/latest/en/std/text/compose/format.html), [entry](https://doc.melodium.tech/latest/en/std/data/string_map/entry.html), [not](https://doc.melodium.tech/latest/en/std/ops/bin/not.html), [exact](https://doc.melodium.tech/latest/en/std/text/compare/exact.html), [unwrapOr](https://doc.melodium.tech/latest/en/std/ops/option/unwrapOr.html), [toString](https://doc.melodium.tech/latest/en/std/conv/toString.html)

## Runtime behaviour

1. Lines are extracted the same way as in examples 03/04/07 (`split` + `flatten` + `trim`, blanks dropped).
2. Each line is parsed with `toJson` into a `Json` value and fed to `process`, which calls the JS `grade(value)` function defined in the `Grader` model's `code`. Inside JS, `value` is the parsed object (`input.score`, `input.name`), exactly the field access that plain `json` treatments cannot do (see 06 and 07, which both work around this).
3. `process` returns `Option<Json>` (`none` if the code throws or returns something that cannot convert to JSON); `unwrapOr` supplies a fallback so the pipeline never stalls on one bad record.
4. Each result is logged and written to `grades.txt`, one JSON object per line, using the same "`entry` + `format` + `\n`" idiom as building any other text report in this tutorial.

### Key Mélodium patterns used

- **The `JavaScriptEngine` model loads code once**: `code` in the model definition is compiled at startup; `process`'s own `code` parameter (`"grade(value)"`) is just the expression evaluated per item, so the actual transform logic is written once and reused across every request/line/track without recompiling.
- **`${{...}}` raw block strings for JS source**: the `grade` function contains newlines and no problematic characters here, but raw blocks are the standard way to embed any multiline code without escaping (see the reference guide for details).
- **JS is the escape hatch for structured JSON access**: when a task needs to read or build specific fields of a JSON object/array, reach for `javascript::process` rather than trying to assemble it from `json` package primitives alone.

Next: [09_process_pipeline](../09_process_pipeline/) introduces running external commands as part of a graph.
