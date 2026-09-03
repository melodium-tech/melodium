# AI Voice Assistant: showcase

Not a tutorial step: this combines several `ml` capabilities from across the library into one convincing demo, without pausing to explain every underlying concept; those are covered individually in [`examples/tutorial/`](../../tutorial/).

> **Requirements:** real API keys (an LLM provider for both entrypoints, plus ElevenLabs for `voice`) and, for `voice`, a working microphone. This example is type-checked with `melodium check` but was not run against live providers for this tutorial.

## Two entrypoints

**`chat`**: an HTTP server that streams a remote LLM's response token by token, as it's generated:

```
melodium run Compo.toml chat --api_key sk-...
curl -X POST http://127.0.0.1:8080/chat -d "What is Mélodium?"
```

**`voice`**: microphone → local Whisper (speech-to-text) → remote LLM (streamed reply) → remote text-to-speech → `answer.mp3`:

```
melodium run Compo.toml voice --llm_api_key sk-... --tts_api_key el-...
```

Speech-to-text runs *locally* (a small Whisper model, fetched once from HuggingFace Hub and cached), so raw audio never leaves the machine: only the transcribed text is sent to the LLM, and only the LLM's reply text is sent to the TTS provider.

## How it is built

| Model | Type | Used by |
|---|---|---|
| `server` | `HttpServer` | `chat` |
| `ChatLlm` / `VoiceLlm` | `RemoteLlm` | `chat` / `voice` |
| `WhisperHub` | `HfHub` | `voice` (downloads model weights) |
| `Asr` | `Whisper` | `voice` (local speech-to-text) |
| `Voice` (the model, not the entrypoint) | `RemoteTts` | `voice` |

### Data flow: `voice`

```
HfHub.fetch ──▶ Whisper.load ──▶ recordMono ──▶ Whisper.decode ──▶ RemoteLlm.stream ──┬──▶ log
                                                                                        └──▶ RemoteTts.synthesize ──▶ writeLocal
```

## Notable choices

- **`connection.started` for `chat`**, exactly as established in [06_http_server_api](../../tutorial/06_http_server_api/): the response is gated on the connection being accepted, not on the body stream starting.
- **A model subclass must set every parameter of its base model that has no default**: `RemoteLlm.temperature`/`top_p`/`timeout` have no default value, so `ChatLlm`/`VoiceLlm` set them to `_` (meaning "use the provider's own default") even though the example never overrides them; `melodium check` catches a missing one immediately.
- **No `visionChat` entrypoint here.** The real `ml/remote/llm::visionChat` treatment takes raw image bytes as a single `Block<Vec<byte>>`, but there is currently no generic "collect an entire byte stream into one block" treatment in `std` to build that value from, say, an HTTP-fetched or locally-read image. Reaching into individual JSON/text values has the same kind of limit worked around with the JavaScript engine in [08_javascript_transform](../../tutorial/08_javascript_transform/); collecting a whole byte stream into one block has no such workaround in the current standard library, so it is left out rather than faked.

Back to the [examples index](../../README.md).
