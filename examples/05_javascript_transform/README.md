# JavaScript Transform Service

An HTTP server that applies a configurable JavaScript function to every incoming JSON payload and returns the transformed result. Demonstrates the `JavaScriptEngine` model: the JS function is compiled once at startup and reused across all concurrent requests.

## What it does

- Starts an HTTP server on the configured port (default 8080).
- Every `POST /transform` request body is parsed as JSON and passed to a JavaScript function as `value`.
- The JS function (`transform`) adds a letter grade (`A`–`F`) based on a `score` field.
- The transformed object is JSON-serialised and returned as the response body.

```
melodium run Compo.toml -- --port 8080

curl -X POST http://127.0.0.1:8080/transform \
     -H "Content-Type: application/json" \
     -d '{"name":"Alice","score":42}'
# → {"name":"Alice","score":42,"grade":"F","note":"processed by Mélodium JS engine"}
```

## How it is built

### Models

| Model | Type | Purpose |
|---|---|---|
| `Transformer` | `JavaScriptEngine` | Hosts the grading JS function; compiled once at program start |
| `server` | `HttpServer` | HTTP listener on localhost at chosen port |

The `Transformer` model holds a JavaScript function `transform(input)` that inspects `input.score` and returns a new object with an added `grade` and `note` field.

### Treatments

**`main`** — Entry point. Starts the server, logs readiness, wires each `/transform` connection to `jsTransform`.

**`jsTransform`** — Per-request pipeline:
1. `decode` — bytes to UTF-8 string.
2. `toJson` — string to `Option<Json>`.
3. `unwrap<Json>` — panics on malformed JSON (strict mode; no fallback).
4. `process[engine=engine]` — calls `transform(value)` in the JS engine with the parsed JSON as `value`; returns `Option<Json>`.
5. `unwrapOr<Json>` — falls back to `|null()` if the JS call fails.
6. `toString<Json>` — serialises the result back to a string.
7. `encode` — string back to bytes for the HTTP response.

### Data flow

```
startup → start[server] + logReady

POST /transform per-request track:
  connection.data → bodyTrigger → status 200 + headers
  connection.data → jsTransform
                      decode → toJson → unwrap<Json>
                             → process (JS: transform(value))
                             → unwrapOr<Json>
                             → toString<Json>
                             → encode
                             → response
```

## Runtime behaviour

1. At startup, `Transformer` is instantiated — the JS source is compiled to bytecode. This happens once regardless of how many requests arrive.
2. The HTTP server starts after `startup.trigger`.
3. For each incoming `POST /transform`:
   - A new track is created; `bodyTrigger` collects the byte stream and gates status/headers.
   - The JSON body flows through the transform pipeline: parse → JS eval → serialise → encode.
   - `process` invokes `transform(value)` synchronously in the embedded JS engine.
   - The result is streamed back as the response body.
4. Concurrent requests each get their own track; the shared `Transformer` model handles them without duplication of compilation cost.

### Key Mélodium patterns used

- **`JavaScriptEngine` as a model** — embedding scripting logic inside a model means the interpreter and compiled bytecode are shared across all tracks, unlike creating a new engine per request.
- **`unwrap<Json>()` vs `unwrapOr<Json>()`** — `unwrap` is used before `process` (malformed JSON is treated as a hard error), while `unwrapOr` is used after `process` (JS errors yield a graceful null fallback).
- **`toString<Json>()`** — serialises the JS-produced result without needing an explicit JSON encode step.
