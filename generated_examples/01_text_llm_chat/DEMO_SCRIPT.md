# Demo Script — Text LLM Chat

**Duration:** ~2 min  
**Angle:** "The simplest possible thing — an HTTP server that streams LLM tokens. Ten lines of meaningful code. Nothing hidden."

---

## Setup (before recording)

- Terminal split: left = editor showing `main.mel`, right = blank shell.
- Have `api_key` ready as env var.
- `curl` available.

---

## Beat 1 — Show the whole file (30 s)

Open [main.mel](main.mel). The file is short — show it all at once.

> "This is the entire server. Let's count what matters."

Point to `model ChatLlm`:

> "One model. OpenAI backend, configurable model name, temperature 0.7."

Point to `model server` inside `main`:

> "One HTTP server. Bound to localhost, port from a parameter."

Point to `treatment chat` (lines 66–79):

> "One treatment for the request logic. Decode bytes, stream tokens, encode back. Three lines of connections."

Point to line 63 — `connection.data -> chat.data,data -> connection.data`:

> "This is the whole request pipeline. The byte stream from the connection goes in, comes back out. The LLM is in the middle."

---

## Beat 2 — Start the server (20 s)

```
melodium run Compo.toml -- --api_key "$OPENAI_KEY"
```

- `[server] LLM chat server started` appears.

> "Server up. One log line."

---

## Beat 3 — Send a request (45 s)

```
curl -X POST http://127.0.0.1:8080/chat \
     -d "What is Mélodium in three sentences?"
```

Pause and let the tokens stream in visibly.

> "Token by token, as they arrive from the API. The HTTP response body grows in real time — no buffering, no waiting for completion."

Send a second request with a different question:

```
curl -X POST http://127.0.0.1:8080/chat \
     -d "Give me a haiku about dataflow programming."
```

> "Each request is its own track — isolated, concurrent, no shared state."

---

## Beat 4 — Change the model live (15 s)

Stop the server. Restart with a different model:

```
melodium run Compo.toml -- --api_key "$OPENAI_KEY" --model gpt-4o
```

```
curl -X POST http://127.0.0.1:8080/chat \
     -d "What changed?"
```

> "Model is a parameter. No code change."

---

## Beat 5 — Close (10 s)

Show [main.mel](main.mel) one last time.

> "80 lines. HTTP server, LLM streaming, concurrent requests. This is the floor — everything else builds on this."
