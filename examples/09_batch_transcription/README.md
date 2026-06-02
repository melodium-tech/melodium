# Batch Audio Transcription

A one-shot CLI program that uploads a single audio file to the OpenAI Whisper API and writes the resulting transcript to a text file. Demonstrates the `RemoteStt` model for cloud-based speech-to-text without any local model download.

## What it does

1. Reads a local audio file (WAV, MP3, FLAC, MP4, OGG, WEBM, or any format accepted by Whisper).
2. Streams the bytes to the OpenAI `whisper-1` API.
3. Receives the full transcript as a single string block.
4. Writes the transcript to an output file.
5. Logs completion and exits.

```
melodium run Compo.toml -- \
  --input      meeting.wav \
  --output     transcript.txt \
  --openai_key sk-...
```

## How it is built

### Models

| Model | Type | Purpose |
|---|---|---|
| `Stt` | `RemoteStt` | OpenAI backend, `whisper-1` model |

### Treatments

**`main`** is the entry point. It triggers the file read on startup, passes bytes directly to `transcribe[stt]`, and handles both the success and error paths.

### Data flow

```
startup → logStart
        → readLocal (audio file)
              ↓ data: Stream<byte>
              ↓ transcript: Block<string>  (fan-out)
          ┌───┴────────────────────────────────────┐
          check<string> → logDone                  sttStream: stream<string>
          (Block<void>  → log trigger)              → write.text: Stream<string>
                                                        → writeTextLocal
```

Error paths:
```
readLocal.failed  → readFailed  (log message)
readLocal.errors  → readErrors  (log errors stream)
transcribe.failed → sttFailed   (log message)
transcribe.error  → sttError    (log error)
write.failed      → writeFailed (log message)
write.errors      → writeErrors (log errors stream)
```

## Runtime behaviour

1. `startup` fires; `logStart` logs "reading audio file..."; `readLocal` opens and streams the file bytes.
2. `transcribe[stt]` receives the byte stream and sends the complete audio to the Whisper API. Whisper processes the full file server-side; this is a blocking network call (no streaming).
3. When the API responds, `transcript: Block<string>` carries the full transcript text. This single block fans out to two paths:
   - `check<string>()` discards the value and emits a `Block<void>` to drive `logDone`.
   - `stream<string>()` converts the block to a one-element `Stream<string>` to feed `writeTextLocal.text`.
4. The transcript is written to the output file; on completion the program exits.

### Key Mélodium patterns used

- **`check<T>()`**: discards the value in a `Block<T>` and emits `Block<void>`, useful when only the event matters (here: "transcription succeeded, log it").
- **`stream<T>()`**: converts `Block<T>` to `Stream<T>` to satisfy stream-typed inputs. `writeTextLocal.text` expects `Stream<string>`, but `transcribe.transcript` is `Block<string>`.
- **Fan-out from a single `Block`**: `transcribe.transcript` drives both `check` (for logging) and `sttStream` (for writing) simultaneously using `-->` (multi-arrow fan-out syntax).
- **`RemoteStt` vs local Whisper**: no model download required; the API handles inference, at the cost of network latency and an API key.
