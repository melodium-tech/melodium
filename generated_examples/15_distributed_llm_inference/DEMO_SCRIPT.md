# Demo Script — Distributed LLM Inference

**Duration:** ~3 min  
**Angle:** "The front-end machine has no ML dependency. The model runs on a cloud runner that Mélodium provisions itself. From the outside it looks like a normal streaming HTTP API — the distribution is invisible to the caller."

---

## Setup (before recording)

- Terminal split: left = editor showing `main.mel`, right = blank shell.
- Have `api_token` and `openai_key` ready as env vars.
- `curl` available.

---

## Beat 1 — Show the architecture (40 s)

Open [main.mel](main.mel). Point to the three blocks in order:

**`model Assistant`** (lines 54–60):

> "This is the LLM — GPT-4o-mini via OpenAI. It's declared as a model, but notice: it only exists on the remote runner, not here."

**`treatment server`** — point to `model runner` and `model distributor` (lines 67–72):

> "Two infrastructure models. `DistantEngine` provisions the cloud runner on demand. `DistributionEngine` routes work to it. The HTTP server only starts after both are up."

**`treatment inferText`** (lines 147–162):

> "This is the treatment that runs remotely. It decodes the prompt, calls the LLM, encodes the tokens back to bytes. The front-end machine never touches the `ml` package."

Point to line 97 — `start(params=|map([|entry<string>("openai_key", openai_key)]))`:

> "The API key is passed once at startup through the params map. Every subsequent request reuses it — there's no key in the HTTP call."

---

## Beat 2 — Start the server (1 min)

```
melodium run Compo.toml -- \
  --api_token  "$MEL_TOKEN" \
  --openai_key "$OPENAI_KEY"
```

**Call out the log lines:**

- `[cloud] provisioning LLM runner…` — "Runner being requested from the API."
- `[distrib] LLM runner connected` — "Channel open. The remote `inferText` treatment is live."
- `[server] HTTP server ready` — "Now accepting requests. The whole setup took a few seconds."

---

## Beat 3 — Send requests (1 min)

In a second terminal, send the first request:

```
curl -X POST http://127.0.0.1:8080/chat \
     -H "Content-Type: text/plain" \
     -d "Explain the Mélodium dataflow model in one sentence."
```

> "Tokens streaming back — each one as it arrives from GPT-4o-mini on the runner."

Send a second request immediately, while the first may still be finishing:

```
curl -X POST http://127.0.0.1:8080/chat \
     -H "Content-Type: text/plain" \
     -d "What is a treatment in Mélodium?"
```

> "Two concurrent requests. Each gets its own distribution slot, its own `inferText` invocation. No queuing."

---

## Beat 4 — The key point (15 s)

Point back at the running server terminal:

> "This machine has no GPU, no `ml` package installed, no Python. The LLM is running on a runner that appeared when the program started. The HTTP API is the only surface."

---

## Beat 5 — Close (5 s)

> "Distribution is not an afterthought. It's part of the program."
