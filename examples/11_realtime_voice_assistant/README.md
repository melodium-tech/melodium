# Real-time Voice Assistant

Two-entrypoint program for a voice-driven conversational assistant. Both entrypoints use a local Whisper model for speech recognition; they differ in the LLM backend: one uses a remote GPT-4o API (streaming tokens), the other uses a fully local Mistral 7B model (no API key needed).

## What it does

**`main` entrypoint** uses local STT with a remote LLM (recommended for production):
- Downloads and loads `openai/whisper-tiny` from Hugging Face.
- Records continuously from the microphone.
- Each transcribed segment is forwarded to GPT-4o via the OpenAI API.
- Tokens stream back and are logged in real time.

**`localOnly` entrypoint** runs fully offline:
- Downloads and loads both `openai/whisper-tiny` and `mistralai/Mistral-7B-v0.1`.
- Same microphone recording and Whisper transcription.
- Uses local Mistral `generate` instead of the remote API.
- Requires approximately 14 GB of RAM for Mistral 7B.

```
# Remote LLM (needs API key)
melodium run Compo.toml -- --openai_key sk-...

# Fully local (no API key, needs ~14 GB RAM)
melodium run Compo.toml localonly
```

## How it is built

### Models

| Model | Type | Used in | Purpose |
|---|---|---|---|
| `WhisperHub` | `HfHub` | both | Hugging Face pointer for Whisper Tiny |
| `Asr` | `Whisper` | both | Local Whisper inference engine |
| `RemoteAssistant` | `RemoteLlm` | `main` | GPT-4o, concise voice-assistant prompt, 256 tokens |
| `MistralHub` | `HfHub` | `localOnly` | Hugging Face pointer for Mistral 7B |
| `LocalLlm` | `Mistral` | `localOnly` | Local Mistral engine, temperature 0.7, 200 tokens |

### Treatments

**`remoteAnswer[llm: RemoteLlm]`** wraps the question in `"[Question] {q}"`, calls `llmStream` (token-by-token streaming), and logs any LLM errors.

**`localAnswer[llm: Mistral]`** wraps the question in `"[INST] {q} [/INST]"` (Mistral instruction format), calls `generate`, and streams the generated text.

Both sub-treatments share the same interface (`question: Stream<string>` → `tokens: Stream<string>`), making the entrypoints structurally identical apart from the model backend.

### Data flow: `main`

```
startup → fetchAsr (Whisper weights) → loadAsr
                   ↓ loaded
              record (mic) + decode[whisper]
                   ↓ transcribed: Stream<string>  (fan-out)
              logQuestion ("you")
              remoteAnswer[llm]
                entry("q") → format → llmStream[llm]
                   ↓ token: Stream<string>
              logAnswer ("assistant")
```

### Data flow: `localOnly`

```
startup ──→ fetchAsr → loadAsr
        └─→ fetchLlm → loadLlm (→ logReady when done)
                 ↓ loadAsr.loaded
              record (mic) + decode[whisper]
                   ↓ transcribed: Stream<string>  (fan-out)
              logQuestion ("you")
              localAnswer[llm]
                entry("q") → format → generate[mistral]
                   ↓ tokens: Stream<string>
              logAnswer ("assistant")
```

## Runtime behaviour

1. **`main`**: Only Whisper is fetched and loaded. Recording starts as soon as `loadAsr.loaded` fires. Each transcribed segment immediately triggers a new `llmStream` call; tokens appear on the console before the sentence is complete.

2. **`localOnly`**: Both models are fetched in parallel (`startup.trigger` fans out). Each model loads independently. `logReady` fires when Mistral is loaded (the slower of the two). Recording starts when Whisper is ready (`loadAsr.loaded`), which may be before Mistral finishes loading, but transcribed text only flows into `localAnswer` after `loadLlm.loaded` is received (Mistral must be ready to generate).

3. In both entrypoints, each transcribed string becomes a separate prompt. Whisper decodes 30-second windows; if the user speaks while the LLM is still generating, the transcription is queued and processed in order.

4. No explicit threading, buffering, or async coordination is written: Mélodium's dataflow graph handles concurrency.

### Key Mélodium patterns used

- **`llmStream` vs `generate`**: `llmStream` yields `Stream<string>` of individual tokens (real-time); `generate` yields `Stream<string>` of full response segments (batch). Both appear identical from the caller's perspective thanks to the shared treatment interface.
- **Dual fan-out on `transcribed`**: each transcribed segment goes both to `logQuestion` (echo to console) and to the LLM treatment, without buffering or copying.
- **Entrypoint-level model selection**: choosing between remote and local LLM is done at the entrypoint level, not within a shared treatment. Each entrypoint owns its models, keeping the code straightforward.
