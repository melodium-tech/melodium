# Demo Script — Real-time Voice Assistant

**Duration:** ~3 min 30 s  
**Angle:** "You speak. The pipeline hears you, understands you, and answers — token by token — before you've finished reading the transcript. Two entrypoints: one with a cloud LLM, one fully offline."

---

## Setup (before recording)

- Terminal split: left = editor showing `main.mel`, right = blank shell.
- Microphone working and set as default device.
- Have `openai_key` ready as env var.
- Quiet environment. Speak clearly.
- Pre-download the Whisper model if possible to skip the download in the demo (run once beforehand).

---

## Beat 1 — Show the two entrypoints (30 s)

Open [main.mel](main.mel). Scroll to the top comment block:

> "Two entrypoints. Same microphone input, same Whisper transcription — different LLM backends."

Point to `treatment main` (line 50):

> "First: local Whisper plus remote GPT-4o. Best for production — raw audio never leaves your machine."

Point to `treatment localOnly` (line 110):

> "Second: everything local. Whisper Tiny plus Mistral 7B. No API key, no network after the first download."

Point to `treatment remoteAnswer` and `treatment localAnswer` side by side (lines 172–197):

> "Both sub-treatments have the same interface — `question` in, `tokens` out. The entrypoint chooses which one to wire up. That's the entire difference between cloud and offline."

---

## Beat 2 — Start the remote entrypoint (30 s)

```
melodium run Compo.toml -- --openai_key "$OPENAI_KEY"
```

Log lines appear:
- Model fetch begins.
- `[assistant] ready — speak into the microphone`

> "Whisper Tiny downloads once, then loads. Ready in a few seconds."

---

## Beat 3 — Live conversation (1 min 30 s)

Speak the first question clearly:

> *"What is a dataflow programming language?"*

**On screen:** `[you] What is a dataflow programming language?` appears, then immediately tokens start streaming under `[assistant]`.

> "The transcript appears as Whisper processes each audio window. The LLM starts answering before the window is even closed."

Speak a follow-up question while the answer is still arriving:

> *"Can you give me a one-line example?"*

> "The pipeline is always listening. Each transcribed segment becomes its own independent track — there's no queue, no turn-taking logic written anywhere."

Let the second answer stream in fully.

---

## Beat 4 — Switch to fully local (30 s)

Stop the server. Switch entrypoint:

```
melodium run Compo.toml localOnly
```

Log lines:
- Two parallel model fetches begin (Whisper + Mistral).
- `[assistant] both models ready — listening…`

> "Both models downloaded in parallel — `startup.trigger` fans out to two fetch treatments simultaneously."

Speak one question:

> *"What time is it on the moon?"*

Let Mistral answer locally.

> "No API key. No network. ~14 GB of RAM and the whole pipeline runs on device."

---

## Beat 5 — Show the mic-to-answer wiring (20 s)

Open [main.mel](main.mel), scroll to the `main` treatment body, and point to:

```
record.signal  -> asrDecode.audio
```
and
```
asrDecode.transcribed -> logQuestion.messages
asrDecode.transcribed -> remoteAnswer.question
```

> "The microphone signal flows into Whisper. The transcript fans out — one copy to the log, one to the LLM. That's the entire voice pipeline. No framework, no event loop, no callback."

---

## Beat 6 — Close (10 s)

> "Local or cloud, streaming or batch — it's a parameter, not a rewrite."
