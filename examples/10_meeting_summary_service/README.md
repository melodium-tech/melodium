# Meeting Summary Service

An HTTP server that accepts raw audio uploads, transcribes them using the OpenAI Whisper API, then asks GPT-4o to produce a structured meeting summary, and streams the summary back as the HTTP response. Each request is fully concurrent; multiple uploads are handled simultaneously in independent tracks.

## What it does

- Starts an HTTP server on the configured port (default 8080).
- Every `POST /summarise` request body is treated as raw audio bytes (WAV, MP3, etc.).
- The audio is sent to the Whisper API for transcription.
- The transcript is wrapped in a structured summarisation prompt and sent to GPT-4o.
- The LLM summary streams back token-by-token as the HTTP response.

```
melodium run Compo.toml -- \
  --openai_key sk-... \
  --port 8080

curl -X POST http://127.0.0.1:8080/summarise \
     --data-binary @meeting.wav \
     -H "Content-Type: audio/wav"
```

## How it is built

### Models

| Model | Type | Purpose |
|---|---|---|
| `Stt` | `RemoteStt` | OpenAI Whisper `whisper-1` for transcription |
| `Llm` | `RemoteLlm` | GPT-4o with a meeting-assistant system prompt, 512 tokens, temperature 0.4 |
| `server` | `HttpServer` | HTTP listener on localhost |

The LLM system prompt instructs GPT-4o to produce key decisions, action items, and a one-paragraph overview from the raw transcript.

### Treatments

**`main`** is the entry point. It starts the server, then wires each `/summarise` connection to `summariseRequest`.

**`summariseRequest[stt, llm]`** is the per-request pipeline:
1. `transcribe[stt]` receives audio bytes and returns a `Block<string>` transcript.
2. `stream<string>()` lifts the block to a stream element.
3. `buildSummaryPrompt` wraps the transcript in the structured prompt template.
4. `chat[llm]` streams the summary tokens.
5. `encode` converts text to bytes for the HTTP response.

**`buildSummaryPrompt`** wraps a transcript string in the prompt `"Here is the meeting transcript:\n\n{t}\n\nPlease produce a structured meeting summary."` using `entry` + `format`.

### Data flow

```
startup → start[server] + logReady

POST /summarise per-request track:
  connection.data → bodyTrigger → status 200 + headers
  connection.data → summariseRequest[stt, llm]
                        transcribe[stt]             (Whisper API)
                          ↓ transcript: Block<string>
                        stream<string>()             (Block → Stream)
                          ↓ transcript: Stream<string>
                          entry("t") → format (prompt template)
                          ↓ prompt: Stream<string>
                        chat[llm]                   (GPT-4o)
                          ↓ response: Stream<string>
                        encode
                          ↓ data: Stream<byte>
                      → connection.data (streamed HTTP response)
```

## Runtime behaviour

1. `startup` fires; the HTTP server starts and a readiness log is emitted.
2. For each `POST /summarise` request, Mélodium creates a new **track** that is fully isolated from all other concurrent requests.
3. `bodyTrigger` fires when the first byte of the request body arrives; status 200 and empty headers are sent immediately so the client knows the server accepted the request before the potentially long processing begins.
4. The audio bytes flow into `transcribe[stt]`, which buffers them and sends the complete file to the Whisper API. This is a blocking call: the track waits for the transcript.
5. The transcript `Block<string>` is lifted to `Stream<string>` by `stream<string>()`, then wrapped in the structured prompt template.
6. `chat[llm]` streams summary tokens from GPT-4o as they arrive; `encode` converts each token to bytes; they appear in the HTTP response body progressively.
7. Multiple simultaneous uploads each run their own independent track with their own Whisper and GPT-4o calls; there is no queue and no serialisation.

### Key Mélodium patterns used

- **Per-request concurrency via tracks**: Mélodium's track model means each HTTP connection becomes an isolated execution context. No thread pool or async runtime management is needed.
- **`stream<string>()`**: the Whisper API returns a single transcript block; `stream<string>()` bridges it to the streaming prompt pipeline downstream.
- **Streaming HTTP response**: because `chat` emits tokens progressively and those flow directly to `connection.data`, the client sees the response grow in real time during the LLM generation phase.
- **Temperature 0.4 for summaries**: lower temperature makes the GPT-4o output more focused and consistent for structured summarisation tasks.
