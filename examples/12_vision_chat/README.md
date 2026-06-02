# Vision Chat (URL-based)

Sends image URLs with questions to a vision-capable LLM (GPT-4o). Two entrypoints are available: a CLI one-shot mode that takes a URL and question as arguments, and an HTTP server mode that accepts JSON bodies with `url` and `question` fields.

## What it does

**`main` entrypoint** runs as a CLI one-shot:
- Takes `--image_url`, `--question`, and `--openai_key` as parameters.
- Builds a prompt from the URL and question, sends it to GPT-4o.
- Streams the response to the console log and writes it to an output file.

**`server` entrypoint** runs as an HTTP server:
- Starts a server on the configured port.
- `POST /describe` accepts a JSON body `{"url":"...","question":"..."}`.
- A JavaScript engine parses the JSON and builds the vision prompt.
- The prompt goes to GPT-4o; the streamed response is sent back as the HTTP response.

```
# CLI mode
melodium run Compo.toml -- \
  --image_url "https://example.com/photo.jpg" \
  --question  "What do you see?" \
  --openai_key sk-...

# Server mode
melodium run Compo.toml server -- --openai_key sk-... --port 8080

curl -X POST http://127.0.0.1:8080/describe \
     -H "Content-Type: application/json" \
     -d '{"url":"https://example.com/photo.jpg","question":"Describe this."}'
```

## How it is built

### Models

| Model | Type | Used in | Purpose |
|---|---|---|---|
| `Vision` | `RemoteLlm` | both | GPT-4o, image analyst system prompt, 512 tokens |
| `PromptBuilder` | `JavaScriptEngine` | `server` | Parses JSON body and builds the vision prompt string |
| `server` | `HttpServer` | `server` | HTTP listener |

The `PromptBuilder` JS function `buildPrompt(body)` accepts either a JSON string or object, extracts `url` and `question`, and returns: `"Please analyse the image at this URL: {url}\n\nQuestion: {question}"`.

### Treatments

**`describeUrl[llm]`** is the CLI path. It emits a `StringMap` with `url` and `question` keys, converts it to a stream, formats the prompt via `format`, and calls `chat[llm]`. Output is `description: Stream<string>`.

**`handleDescribe[llm]`** is the server path. It decodes the request bytes, parses JSON, calls JS `buildPrompt`, extracts the string, calls `chat[llm]`, encodes the response, and returns bytes.

### Data flow: `main` (CLI)

```
startup -> logStart
        -> describeUrl[llm](image_url=..., question=...)
               emit<StringMap> -> stream<StringMap>
               -> format("Please analyse the image at: {url}\n\nQuestion: {q}")
               -> chat[llm]
                  | response: Stream<string>
           logDesc (console) + write (description.txt) -> done
```

### Data flow: `server` (HTTP)

```
startup -> start[server] + logReady

POST /describe per-request track:
  connection.data -> bodyTrigger -> status 200 + headers
  connection.data -> handleDescribe[llm]
                       decode -> toJson -> unwrapOr<Json>
                       -> process (JS: buildPrompt(value))
                       -> unwrapOr<Json>
                       -> tryToString<Json>
                       -> unwrapOr<string> (default "")
                       -> chat[llm]
                          | response: Stream<string>
                       encode
                          | data: Stream<byte>
                       -> connection.data
```

## Runtime behaviour

1. **CLI mode**: `startup` fires; `describeUrl` builds the prompt from the constant `image_url` and `question` parameters. An `emit<StringMap>` creates a single-element block, which `stream<StringMap>()` promotes to a stream for `format`. The formatted prompt is sent to `chat[llm]`. Tokens stream back, fan out to the console log and the file write.

2. **Server mode**: For each `POST /describe`, a new track is created. The raw request bytes are decoded to UTF-8, parsed as JSON, and passed to the `PromptBuilder` JavaScript engine. The JS function handles both pre-parsed JSON objects and raw JSON strings (defensive parsing). The returned string drives `chat[llm]`. GPT-4o streams response tokens; `encode` converts each to bytes and they appear progressively in the HTTP response.

3. The `PromptBuilder` model is compiled once at startup and shared across all request tracks, eliminating per-request JS compilation cost.

### Key Mélodium patterns used

- **`emit<StringMap>` + `stream<StringMap>()`**: the CLI path uses a constant `StringMap` to feed `format`, which expects `Stream<StringMap>`. `emit` creates a single-shot block; `stream` promotes it.
- **JavaScript for dynamic prompt construction**: rather than a fixed template, `PromptBuilder` can handle varied JSON shapes and apply fallback defaults when fields are missing.
- **`tryToString<Json>()`**: the JS engine returns `Option<Json>`; `tryToString` attempts to extract the inner string from the JSON value, yielding `Stream<Option<string>>`. The `.into` output name carries this value.
- **Two entrypoints, one model type**: both `main` and `server` use the same `Vision` model type but instantiate it independently with their own `openai_key`.
