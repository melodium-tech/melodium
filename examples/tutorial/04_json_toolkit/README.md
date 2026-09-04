# 04: JSON Toolkit

**Concepts introduced:** parsing & validating JSON (`json`), `Option` unwrapping, building structured data (`StringMap` → JSON object).

Reads one record per line (a mix of valid JSON values and garbage), separates the invalid lines, classifies the valid ones as JSON objects or plain scalars (strings, numbers, booleans, arrays), and writes a small JSON summary of the counts.

## What it does

```
melodium run Compo.toml --input_file records.txt
```

With the shipped `records.txt` (7 lines: strings, a number, a bool, two objects, one array, and one line of garbage), it logs each classified record and writes `summary.json`:

```json
{"invalid":"1","objects":"2","scalars":"4"}
```

## How it is built

No models here: JSON parsing and validation are stateless treatments.

### Data flow

```
readTextLocal ──▶ lines ──▶ validate ──▶ filter ──┬──▶ toJson ──▶ isObject ──▶ filter ──┬──▶ objects
                                                    └── (invalid, logged)                 └──▶ scalars
                                                                                                  ↓
                                                                              counts ──▶ StringMap ──▶ JSON ──▶ writeTextLocal
```

## Runtime behaviour

1. Lines are extracted the same way as in example 03 (`split` + `flatten` + `trim`, blank lines dropped).
2. `json::validate` checks each line without parsing it; `filter` splits the stream into valid text (`accepted`) and garbage (`rejected`, logged as-is).
3. Only the valid text reaches `toJson`, so parsing never fails here: but `toJson` still returns `Stream<Option<Json>>` by design (it has no way to know that from the type alone), so `unwrapOr` is used to get a plain `Stream<Json>`.
4. `json/value::isObject` classifies each `Json` value; a second `filter` splits objects from scalars (strings, numbers, booleans, arrays: anything that is not a JSON object).
5. Three totals (objects, scalars, invalid lines) are combined into one `StringMap`, converted to a JSON object with `fromStringMap`, serialised with `toString<Json>`, and written to `summary.json`.

### Key Mélodium patterns used

- **`validate` before `toJson`**: checking validity first avoids ever having to handle a parse failure downstream; the `Option` returned by `toJson` still has to be unwrapped, but it is guaranteed to always be `some`.
- **Classifying with a boolean predicate + `filter`**: the same shape as example 03's regex matching, just with `json/value::isObject` instead of `regex::matches`. Most "does this satisfy X" library treatments are designed to plug directly into `filter.select`.
- **`fromStringMap`**: the direct way to build a JSON *object* out of Mélodium data; every value becomes a JSON string, which is enough for a summary report (for richer JSON, numbers or nested objects, build it with the `json/value::from*` functions/treatments individually).
- **`finalCount<T>`**: the same counter from example 03, reused here at two different types (`Json` and `string`) in the same file, without changing a single line of its body.

Next: [05_http_client](../05_http_client/) introduces calling a remote HTTP API.
