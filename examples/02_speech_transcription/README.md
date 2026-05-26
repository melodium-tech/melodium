# Speech Transcription

Two-entrypoint program that transcribes audio to text using a local Whisper model downloaded from Hugging Face. One entrypoint records from the microphone continuously; the other reads a file once and exits.

## What it does

**`main` entrypoint** — live microphone transcription:
- Downloads `openai/whisper-tiny` from Hugging Face (cached after first run).
- Loads the model into a local `Whisper` engine.
- Records mono audio from the default microphone.
- Decodes each audio window into a transcript string, appends it to a file, and logs it.

**`fromFile` entrypoint** — audio file transcription:
- Same model download and load sequence.
- Reads a local audio file (WAV, MP3, FLAC, etc.) instead of recording.
- Decodes the audio container format before passing samples to Whisper.
- Writes the transcript to an output file (overwrite, not append).

```
# Live mic
melodium run Compo.toml -- --output transcription.txt

# From file
melodium run Compo.toml fromFile --input_file speech.wav --output transcription.txt
```

## How it is built

### Models

| Model | Type | Purpose |
|---|---|---|
| `Hub` | `HfHub` | Hugging Face repo pointer (`openai/whisper-tiny`) |
| `Speech` | `Whisper` | Local inference engine for Whisper |

### Treatments

**`main`** — microphone path:
1. `fetch` downloads safetensors weights and tokenizer from the Hub.
2. `load` initialises the Whisper engine with those weights.
3. Once loaded, `recordMono` starts microphone capture and `decode` starts listening for audio signals.
4. `decode.transcribed` fans out to `logInfos` (console) and `writeTextLocal` (file, append mode).

**`fromFile`** — file path:
1. Same fetch + load sequence.
2. `readLocal` reads the audio file bytes.
3. `decodeMono` parses the audio container (WAV/MP3/…) into raw samples.
4. Samples are fed to `decode[whisper]`; transcription fans out to log and file (overwrite mode).

### Data flow — `main` (microphone)

```
startup → fetch (Hub) → load (Whisper engine)
                             ↓
                        recordMono ──→ decode[whisper] → logInfos
                                                       → writeTextLocal
```

### Data flow — `fromFile`

```
startup → fetch (Hub) → load (Whisper engine)
                             ↓
                        readLocal → decodeMono → decode[whisper] → logInfos
                                                                 → writeTextLocal
```

## Runtime behaviour

1. Both entrypoints start by fetching the model. `fetch` emits `safetensors` and `tokenizer` as block values when each download completes.
2. `load` blocks until both are received, then emits `loaded` once the engine is ready.
3. **`main`**: `loaded` simultaneously starts `recordMono` and arms `decode`. The microphone streams audio frames; each processed 30-second window yields a `transcribed: Block<string>` value.
4. **`fromFile`**: `loaded` triggers `readLocal`; bytes flow through `decodeMono` → `decode`. After a single transcript block is emitted, the pipeline completes and the process exits.
5. All error paths (`fetch.failed`, `load.failed`, `record.failed`, etc.) are individually logged but do not abort the pipeline.

### Key Mélodium patterns used

- **Model fan-out with `loaded`** — the same `Block<void>` drives both `record.trigger` and `decode.ready`, ensuring the mic starts only after the model is ready.
- **Dual fan-out on `transcribed`** — `-->` (multi-dash) connects one output to two inputs: `logInfos` and `writeTextLocal`, both receive every transcript string independently.
- **`decodeMono` container parsing** — sits between raw file bytes and the Whisper engine, handling format detection transparently.
