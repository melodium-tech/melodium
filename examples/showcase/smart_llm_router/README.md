# Smart LLM Router: showcase

Not a tutorial step: a JavaScript decision function reads each incoming prompt, estimates how complex it actually is, and routes it to one of three pre-configured `RemoteLlm` tiers, instead of always paying for the biggest model and the largest response budget regardless of what was actually asked.

> **Requirements:** a real LLM provider API key. The server, routing, and the actual outbound request to the provider were all run and verified end to end with `melodium run` and real `curl` requests (see *How it was checked* below); only the provider's own reply was not observed, since a real key was not available for this tutorial.

## What it does

```
melodium run Compo.toml --api_key sk-...

curl -X POST http://127.0.0.1:8080/chat -d "What year is it?"
curl -X POST http://127.0.0.1:8080/chat -d "Can you summarize the main differences between REST and GraphQL APIs?"
curl -X POST http://127.0.0.1:8080/chat -d "Explain, step by step, how a hash map resizes, and compare it to a B-tree's rebalancing cost."
```

Verified with `melodium run` against the real server, HTTP requests included (see *How it was checked* below): the first request (short, factual) is routed to the `economy` tier; the second (moderate length, one complexity signal) to `standard`; the third (three distinct complexity signals: "explain", "step by step", "compare") to `premium`. The server log prints the router's decision and its reasoning for every request:

```json
{"complexity_score":-1.0,"estimated_input_tokens":4.0,"reason":"short, simple request","tier":"economy","word_count":4}
{"complexity_score":1.0,"estimated_input_tokens":18.0,"reason":"moderate length or complexity","tier":"standard","word_count":11}
{"complexity_score":3.0,"estimated_input_tokens":24.0,"reason":"long or explicitly complex request","tier":"premium","word_count":17}
```

`complexity_score` and `estimated_input_tokens` (built with arithmetic: `+=`, `Math.min`, `Math.ceil`) print with a trailing `.0`, even for whole numbers; `word_count` (read straight off `Array.length`) does not. JavaScript has no separate integer type, so this comes from how the embedded JS engine happens to represent each value internally, not from any rounding choice on Mélodium's side. Do not assume every numeric field here parses cleanly as an integer downstream without checking.

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `server` | `HttpServer` | Front-end HTTP listener |
| `router` | `JavaScriptEngine` | Holds `decide()` and `pickTier()`, compiled once at startup |
| `economyLlm` / `standardLlm` / `premiumLlm` | `RemoteLlm` | Three distinct `(model, max_tokens)` presets |

### Data flow

```
POST /chat body ──▶ collapse to one block ──▶ decide() (JS) ──▶ log full decision
                                          │                  └─▶ pickTier() (JS) ──▶ tier name
                                          │
                                          ├──▶ [tier == economy]  ──▶ economyLlm.stream  ──┐
                                          ├──▶ [tier == standard] ──▶ standardLlm.stream ──┼─▶ merge ──▶ encode ──▶ response
                                          └──▶ [tier == premium]  ──▶ premiumLlm.stream  ──┘
```

## Runtime behaviour

1. Mélodium's `RemoteLlm` sets `model` and `max_tokens` once per model instance, not per request, so "optimise the token budget for this request" cannot mean "compute an arbitrary number every time"; it means "pick the right one of a few pre-defined `(model, budget)` tiers": `economyLlm` (small, fast model, 200-token budget), `standardLlm` (600 tokens), `premiumLlm` (most capable model, 1500 tokens).
2. The request body is collapsed to a single `Block<string>` (the `trigger.last` idiom from example 03), wrapped as `Json` with `fromString`, and passed to the JS `decide()` function, which returns a JSON object with the chosen `tier`, a word/token estimate, and a human-readable `reason`. That whole object is logged server-side for every request.
3. A second, tiny JS call, `pickTier(decision)`, projects just the `tier` field back out; there is still no field-by-field access into a parsed `Json` value outside JavaScript (see 08_javascript_transform), so chaining a second `process` call on the first call's own output is the way to read one field out of a result you already computed in JS.
4. The prompt is routed with three `equalTo` + `filterBlock` gates, one per tier (the block-level counterparts of `filter`, used everywhere else in this tutorial on streams). Exactly one gate's `accepted` output actually carries the prompt; the other two close empty. Because an LLM `stream` treatment fed an empty prompt stream never calls the provider at all, the two tiers *not* chosen cost nothing, not even a request.
5. The three (mostly empty) token streams are combined with two `merge`s into one; since only one branch ever produced anything, the merged stream is just that branch's output.

### How it was checked

Run with `melodium run Compo.toml --api_key sk-fake-test-key` and all three `curl` requests from *What it does* above, against the real HTTP server on a real port, with a deliberately invalid key. All three requests routed to the tier the prose above claims, the router's full decision was logged for each, and each tier's `RemoteLlm` genuinely attempted its outbound request and got a real `401 Unauthorized` back from the provider, exactly what a request that got everything right up to the point of needing a real credential should do. That confirms the routing, the HTTP wiring, and the request assembly all work; only the provider's own successful reply was never observed, since that needs a real key.

### Key Mélodium patterns used

- **A model's constant parameters are the unit of "configuration", not the request.** When something needs to vary per request but the model only exposes it as a fixed parameter, the answer is several model instances, not a dynamic parameter.
- **JS as a two-step pipeline**: one `process` call to compute a whole decision object, a second `process` call (same engine, same compiled functions) to project one field out of it. Chaining calls like this is cheaper than re-deriving the projection outside JS, which usually is not possible anyway (see point 3 above).
- **`equalTo` + `filterBlock`**: the block-level shape of the `exact`/`filter` combination used on streams throughout this tutorial, for routing a single request rather than filtering a sequence.
- **An empty prompt stream costs nothing.** Feeding all three tiers and letting the unchosen ones receive zero items is simpler than building a true N-way dynamic dispatch, and is free: an LLM treatment that never receives a prompt never calls the provider.
- **Verify logic in isolation before wiring it into something you can't easily test** (here, real API calls): the same principle as `melodium run` over `melodium check` elsewhere in this tutorial, applied to a piece of business logic rather than a library treatment.

Back to the [examples index](../../README.md).
