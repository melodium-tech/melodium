# AI Voice Assistant

Combines several `ml` capabilities from across the library: remote LLM chat, local speech-to-text, and remote text-to-speech.

> **Requirements:** real API keys (an LLM provider for both entrypoints, plus ElevenLabs for `voice`) and, for `voice`, a working microphone.

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

*Optional: add `--api-report` and a Mélodium Services API token (`MELODIUM_API_TOKEN`) to see this run's full trace on [Cadence.CI](https://cadence.ci/).*

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

### Reference

- **`encoding`**: [decode](https://doc.melodium.tech/latest/en/encoding/decode.html), [encode](https://doc.melodium.tech/latest/en/encoding/encode.html)
- **`fs`**: [writeLocal](https://doc.melodium.tech/latest/en/fs/local/writeLocal.html)
- **`http`**: [HttpServer](https://doc.melodium.tech/latest/en/http/server/HttpServer.html), [start](https://doc.melodium.tech/latest/en/http/server/start.html), [connection](https://doc.melodium.tech/latest/en/http/server/connection.html), [|post](https://doc.melodium.tech/latest/en/http/method/|post.html), [|ok](https://doc.melodium.tech/latest/en/http/status/|ok.html), [HttpStatus](https://doc.melodium.tech/latest/en/http/status/HttpStatus.html)
- **`ml`**: [HfHub](https://doc.melodium.tech/latest/en/ml/repos/hf/HfHub.html), [fetch](https://doc.melodium.tech/latest/en/ml/repos/hf/fetch.html), [Whisper](https://doc.melodium.tech/latest/en/ml/models/whisper/Whisper.html), [load](https://doc.melodium.tech/latest/en/ml/models/whisper/load.html), [whisperDecode](https://doc.melodium.tech/latest/en/ml/models/whisper/decode.html), [RemoteLlm](https://doc.melodium.tech/latest/en/ml/remote/llm/RemoteLlm.html), [stream](https://doc.melodium.tech/latest/en/ml/remote/llm/stream.html), [RemoteTts](https://doc.melodium.tech/latest/en/ml/remote/tts/RemoteTts.html), [synthesize](https://doc.melodium.tech/latest/en/ml/remote/tts/synthesize.html)
- **`net`**: [|localhost_ipv4](https://doc.melodium.tech/latest/en/net/ip/|localhost_ipv4.html), [|from_ipv4](https://doc.melodium.tech/latest/en/net/ip/|from_ipv4.html)
- **`record`**: [recordMono](https://doc.melodium.tech/latest/en/record/audio/recordMono.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logInfos](https://doc.melodium.tech/latest/en/std/engine/log/logInfos.html), [logError](https://doc.melodium.tech/latest/en/std/engine/log/logError.html), [logErrorMessage](https://doc.melodium.tech/latest/en/std/engine/log/logErrorMessage.html), [logErrors](https://doc.melodium.tech/latest/en/std/engine/log/logErrors.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [StringMap](https://doc.melodium.tech/latest/en/std/data/string_map/StringMap.html), [|map](https://doc.melodium.tech/latest/en/std/data/string_map/|map.html), [|wrap](https://doc.melodium.tech/latest/en/std/ops/option/|wrap.html)

## Notable choices

- **`connection.started` for `chat`**, exactly as established in [06_http_server_api](../../tutorial/06_http_server_api/): the response is gated on the connection being accepted, not on the body stream starting.
- **A model subclass must set every parameter of its base model that has no default**: `RemoteLlm.temperature`/`top_p`/`timeout` have no default value, so `ChatLlm`/`VoiceLlm` set them to `_` (meaning "use the provider's own default") even though the example never overrides them; `melodium check` catches a missing one immediately.

Back to the [examples index](../../README.md).
