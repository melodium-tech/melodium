# Voice Q&A (Fully Local)

A fully offline voice question-answering pipeline. Speech is captured from the microphone, transcribed by a local Whisper Tiny model, and answered by a local Mistral 7B model. No API keys or network access required after the initial model download.

## What it does

- Downloads `openai/whisper-tiny` and `mistralai/Mistral-7B-v0.1` from Hugging Face (cached after first run).
- Loads both models into local inference engines in parallel.
- Records mono audio continuously from the default microphone.
- Each transcribed segment is wrapped in a Mistral `[INST]…[/INST]` prompt and sent to the language model.
- Generated text is logged to the console and appended to an output file.

```
melodium run Compo.toml -- --output qa.txt
```

## How it is built

### Models

| Model | Type | Purpose |
|---|---|---|
| `WhisperHub` | `HfHub` | Hugging Face pointer for `openai/whisper-tiny` |
| `MistralHub` | `HfHub` | Hugging Face pointer for `mistralai/Mistral-7B-v0.1` |
| `Asr` | `Whisper` | Local Whisper inference engine |
| `Llm` | `Mistral` | Local Mistral inference engine (temperature 0.7, top_p 0.9, 256 tokens) |

### Treatments

**`main`** — Entry point. Fetches both models in parallel (`startup.trigger` fans out to `fetchAsr` and `fetchLlm`). Each fetch drives its own `load`. Recording and decoding begin once Whisper is ready; `promptLlm` is called for each transcribed segment.

**`promptLlm[llm: Mistral]`** — Wraps each question string in a Mistral instruction template using `entry` + `format`, then calls `generate[mistral=llm]`. Outputs the generated text as a stream of strings.

### Data flow

```
startup ──→ fetchAsr (Whisper weights) → loadAsr
        └─→ fetchLlm (Mistral weights) → loadLlm
                                              ↓ loaded (logReady)

loadAsr.loaded ──→ record (microphone)
              └──→ decode[whisper]
                       ↓ transcribed: Stream<string>
                   promptLlm[mistral]
                     entry("q") → format("[INST] {q} [/INST]") → generate
                       ↓ answer: Stream<string>
                   logInfos ("answer") + writeTextLocal (qa.txt, append)
```

## Runtime behaviour

1. `startup` fires both model fetches simultaneously. Each fetch downloads independently; the pipeline does not wait for both before starting either load — loads start as soon as their own fetch completes.
2. `loadAsr.loaded` gates both `recordMono` (starts the mic) and `asrDecode.ready` (arms Whisper). Whisper begins processing audio immediately.
3. Each 30-second audio window yields a `transcribed: Block<string>`. This single block value is forwarded directly to `promptLlm.question` as a `Stream<string>` element.
4. `promptLlm` wraps the question in the `[INST]…[/INST]` format required by Mistral instruction-tuned models, then calls `generate`. Generated tokens stream out progressively.
5. Generated text fans out to `logInfos` (real-time console display) and `writeTextLocal` (appended to `qa.txt`).
6. `logReady` fires only after `loadLlm.loaded` — the "both models ready" message appears when the slower of the two loads finishes.

### Key Mélodium patterns used

- **Parallel model fetching** — `startup.trigger` fans out to two `fetch` calls; both downloads proceed concurrently without explicit threading.
- **`entry` + `format` for prompt templating** — a `StringMap` entry wraps the question, then `format` substitutes `{q}` into the Mistral instruction template without string concatenation primitives.
- **`loadAsr.loaded` double fan-out** — the same `Block<void>` arms both the microphone and the decoder, ensuring the decoder is ready before audio arrives.
- **Fully local execution** — after initial download, the pipeline runs entirely on-device: no API calls, no network latency, no usage costs.
