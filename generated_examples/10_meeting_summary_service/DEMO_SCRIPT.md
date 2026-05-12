# Demo Script — Meeting Summary Service

**Duration:** ~3 min  
**Angle:** "Upload a recording, get a structured summary streamed back. Multiple uploads handled at the same time. The concurrency model is the program structure — there's nothing to configure."

---

## Setup (before recording)

- Terminal split: left = editor showing `main.mel`, right = blank shell.
- Have `openai_key` ready as env var.
- Two short audio files ready: `meeting_a.wav` (1–2 min of spoken content) and `meeting_b.wav` (different content).
- `curl` available.

---

## Beat 1 — Show the request pipeline (40 s)

Open [main.mel](main.mel). Point to `treatment summariseRequest` (lines 91–116):

> "This is everything that happens per request. Four stages, sequential."

Walk through each step:

- `transcribe[stt]` — "Audio bytes go to Whisper API. One blocking call — returns the full transcript."
- `stream<string>()` — "The transcript is a single block. `stream` lifts it to a stream element so it can feed the prompt builder."
- `buildSummaryPrompt` — "Wraps the transcript in a structured prompt: key decisions, action items, overview."
- `chat[llm]` → `encode` — "GPT-4o streams the summary. Each token is encoded and sent back as the HTTP response grows."

Point to the `main` treatment body — specifically the lack of any concurrency code:

> "There's no thread pool here. No worker queue. Each HTTP connection creates its own track — an isolated execution context. Concurrency is the default."

---

## Beat 2 — Start the server (20 s)

```
melodium run Compo.toml -- --openai_key "$OPENAI_KEY"
```

- `[service] meeting summary service ready`

> "Ready."

---

## Beat 3 — Single upload (40 s)

```
curl -X POST http://127.0.0.1:8080/summarise \
     --data-binary @meeting_a.wav \
     -H "Content-Type: audio/wav"
```

> "Audio uploaded. Whisper processing... now GPT-4o streaming the summary."

Let it complete. The structured output — decisions, action items, overview — appears token by token.

> "The format comes from the system prompt in the model declaration. Change it once, every request uses it."

---

## Beat 4 — Concurrent uploads (40 s)

Open two shells side by side. Fire both at the same time:

**Shell 1:**
```
curl -X POST http://127.0.0.1:8080/summarise \
     --data-binary @meeting_a.wav \
     -H "Content-Type: audio/wav"
```

**Shell 2 (immediately after):**
```
curl -X POST http://127.0.0.1:8080/summarise \
     --data-binary @meeting_b.wav \
     -H "Content-Type: audio/wav"
```

Watch both responses stream back simultaneously.

> "Two requests, two independent Whisper calls, two independent GPT-4o streams — happening at the same time. No configuration. No worker count. The program structure is the concurrency model."

---

## Beat 5 — Show the system prompt (20 s)

Open [main.mel](main.mel), scroll to the `Llm` model (lines 54–59):

> "The system prompt is here, in the model declaration. This is the only place it lives. Structured output is a prompt engineering decision, not a framework feature."

Point to `temperature = 0.4`:

> "Lower temperature for summaries. Higher for chat. It's a field."

---

## Beat 6 — Close (10 s)

> "An audio upload endpoint, a transcription call, a structured LLM prompt, a streaming response — four treatments, one file, handles any number of concurrent meetings."
