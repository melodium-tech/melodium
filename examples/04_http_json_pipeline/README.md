# HTTP + JSON Pipeline

A one-shot CLI program that fetches a list of posts from a public JSON API, parses the response, and writes the result to a local file. Demonstrates chaining an HTTP GET client call with JSON parsing in a linear dataflow pipeline.

## What it does

1. Fires a single `GET` request to `https://jsonplaceholder.typicode.com/posts`.
2. Streams the response bytes through a JSON parser.
3. Converts the parsed JSON value to its string representation.
4. Writes the string to a local output file (default `results.txt`).
5. Logs a "done" message and exits.

```
melodium run Compo.toml -- --output results.txt
```

## How it is built

This example has no models: it is a pure dataflow pipeline with no stateful resources.

### Treatments

**`main`** is the entry point. It triggers both `logFetch` and `fetchPosts` simultaneously on startup, then pipes the response bytes into `parsePosts`, writes the output, and logs completion.

**`parsePosts`** is a reusable sub-pipeline that receives raw bytes and produces a string:
- `decode` converts UTF-8 bytes to a text string.
- `toJson` parses text into `Option<Json>`.
- `unwrapOr<Json>` provides `|null()` as fallback if parsing fails.
- `toString<Json>` converts the `Json` value to its string form.

### Data flow

```
startup ──→ logFetch ("fetching posts…")
        └─→ fetchPosts (GET request)
                 ↓ data: Stream<byte>
              decode → toJson → unwrapOr<Json> → toString<Json>
                 ↓ text: Stream<string>
            writeTextLocal (output file)
                 ↓ finished
            logDone ("done")
```

Error paths:
```
fetchPosts.failed → fetchFailed (log message)
fetchPosts.error  → fetchError  (log error)
```

## Runtime behaviour

1. `startup` fires; both the info log and the HTTP GET are triggered in parallel.
2. `fetchPosts` opens a connection to `jsonplaceholder.typicode.com` and streams response bytes.
3. Bytes flow through `parsePosts`: decoded to UTF-8, parsed as JSON, converted to a string. Each chunk arrives as it streams in.
4. `writePosts` appends (or creates) the output file as string chunks arrive.
5. When the write completes (`finished: Block<void>`), `logDone` emits a single log line.
6. The program exits naturally once all outputs are satisfied.

### Key Mélodium patterns used

- **No models**: demonstrates that simple pipelines need no stateful resources.
- **`toString<Json>()`**: the `Json` type implements `ToString`, so this generic treatment can serialise any parsed JSON back to text.
- **`unwrapOr<Json>(default=|null())`**: graceful JSON parse failure handling; if the API returns malformed data, the pipeline continues with a JSON null value instead of stalling.
- **Linear chain syntax**: `Self.data -> decode.data,text -> toJson.text,json -> unwrapOr.option,value -> toString.value,into -> Self.text` shows how multiple treatments are chained in a single connection statement.
