# Text LLM Chat

An HTTP server that accepts plain-text prompts over POST and streams token-by-token LLM responses back to the caller using a remote OpenAI-compatible backend.

## What it does

- Starts an HTTP server on a configurable port (default 8080).
- Every `POST /chat` request body is decoded from bytes to UTF-8 and sent to a remote LLM.
- The LLM response is streamed back token by token: the HTTP response body grows as each token arrives, rather than waiting for full completion.
- LLM errors are logged and do not crash the server; subsequent requests are handled normally.

```
curl -X POST http://127.0.0.1:8080/chat -d "What is Mélodium?"
```

## How it is built

### Models

| Model | Type | Purpose |
|---|---|---|
| `ChatLlm` | `RemoteLlm` | OpenAI backend, configurable model name, 512-token limit, temperature 0.7 |
| `server` | `HttpServer` | HTTP listener bound to localhost at the chosen port |

Both models are instantiated once at program start and shared across all concurrent request tracks.

### Treatments

**`main`** — Entry point. Starts the HTTP server and wires each incoming connection to the `chat` treatment.

**`chat`** — Per-request pipeline. Decodes the raw request bytes to text, calls the remote LLM using `llmStream`, and encodes the token stream back to bytes for the HTTP response. LLM errors are forwarded to `logErrors`.

### Data flow inside `chat`

```
request bytes  →  decode  →  llmStream (token stream)  →  encode  →  response bytes
                                  ↓ (errors)
                              logErrors
```

## Runtime behaviour

1. At startup, Mélodium instantiates the `ChatLlm` and `HttpServer` models, then fires `startup.trigger`.
2. `start` binds the server socket; `logStarted` emits a log line.
3. For each incoming `POST /chat`:
   - The HTTP framework creates a new **track** (an isolated execution context).
   - `bodyTrigger` fires when the first byte arrives; status 200 and empty headers are sent immediately so the response begins streaming.
   - The request bytes flow through `decode` → `llmStream` → `encode` → back into `connection.data`.
   - Tokens appear in the HTTP response body as soon as the LLM emits them.
4. The server remains live indefinitely; each request track is independent and concurrent.

### Key Mélodium patterns used

- **`trigger<byte>()`** — separates the incoming byte stream arrival from the trigger that sends headers/status, allowing status 200 to be emitted before the body is fully read.
- **`llmStream` vs `chat`** — `llmStream` outputs individual tokens (`Stream<string>`) for real-time streaming; `chat` would collect a complete response.
- **Fan-out on `bodyTrigger.start`** — the same `Block<void>` signal drives both `status.trigger` and `headers.trigger` in parallel.
