# Full Remote Voice Pipeline

A complete speech-in / speech-out pipeline using three cloud APIs: OpenAI Whisper for transcription, GPT-4o for response generation, and ElevenLabs for speech synthesis. Reads an audio file, answers the question it contains, and writes the synthesised answer as an audio file.

## What it does

1. Reads an audio file from disk.
2. Sends the audio bytes to the OpenAI Whisper API (`whisper-1`) and receives a text transcript.
3. Wraps the transcript in a prompt and sends it to GPT-4o; receives a streamed text response.
4. Sends each response string to the ElevenLabs TTS API; receives synthesised audio bytes.
5. Writes the audio bytes to an output file (default `answer.mp3`).

```
melodium run Compo.toml -- \
  --input_file  question.wav \
  --output_file answer.mp3 \
  --openai_key  sk-... \
  --elevenlabs_key el-... \
  --elevenlabs_voice JBFqnCBsd6RMkjVDRZzb
```

## How it is built

### Models

| Model | Type | Purpose |
|---|---|---|
| `Stt` | `RemoteStt` | OpenAI Whisper API, `whisper-1` model |
| `Llm` | `RemoteLlm` | GPT-4o, voice assistant system prompt, 256 tokens, temperature 0.7 |
| `Tts` | `RemoteTts` | ElevenLabs `eleven_multilingual_v2`, configurable voice ID |

### Treatments

**`main`** is the entry point. It sequences file read → STT → LLM → TTS → file write as a linear pipeline, delegating each stage to a sub-treatment.

**`sttTranscribe[stt]`** sends audio bytes to `transcribe[stt]`. The result is a `Block<string>`; `stream<string>()` converts it to `Stream<string>` for downstream streaming inputs.

**`llmRespond[llm]`** wraps each transcript string in a user prompt via `entry` + `format`, then calls `chat[llm]` which streams response tokens.

**`ttsSpeak[tts]`** passes each response string to `synthesize[tts]`, which streams back raw audio bytes.

### Data flow

```
startup → read (audio file)
              ↓ data: Stream<byte>
            sttTranscribe[stt]
              transcribe → stream<string>
              ↓ transcript: Stream<string>
            llmRespond[llm]
              entry("q") → format → chat[llm]
              ↓ answer: Stream<string>
            ttsSpeak[tts]
              synthesize[tts]
              ↓ audio: Stream<byte>
            writeLocal (output file)
              ↓ completed
            logDone
```

## Runtime behaviour

1. `startup` fires; `logStart` emits a log line; `read` begins reading the audio file bytes.
2. The byte stream flows directly into `sttTranscribe`. `transcribe` buffers all audio bytes and sends them to the Whisper API in one request (block-level API). When the transcript arrives as a `Block<string>`, `stream<string>()` lifts it into a one-element `Stream<string>`.
3. The transcript string is wrapped in the prompt template `"User asked: {q}\nPlease answer helpfully."` by `llmRespond`. `chat[llm]` streams response tokens as they arrive from the GPT-4o API.
4. Each response token (string) is forwarded to `ttsSpeak`. `synthesize` sends it to the ElevenLabs API and streams audio bytes back.
5. Audio bytes are written to the output file. When writing completes, `logDone` fires and the program exits.

### Key Mélodium patterns used

- **`stream<string>()`**: bridges the Block/Stream boundary: `transcribe` returns a single `Block<string>`, but downstream treatments (LLM prompt, TTS) expect `Stream<string>`. The `stream` treatment converts one into the other.
- **Three-API chaining without threading**: the three cloud API calls form a natural dataflow chain. Mélodium schedules each call as soon as its upstream data is available, without any explicit async code.
- **`entry` + `format` prompt construction**: builds structured prompts from dynamic values using a template string, avoiding string interpolation or concatenation.
- **Error isolation by stage**: each sub-treatment (`sttTranscribe`, `llmRespond`, `ttsSpeak`) handles its own error logging, keeping error handling co-located with the stage that can fail.
