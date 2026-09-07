# Smart LLM Router: showcase

Not a tutorial step: a JavaScript decision function reads each incoming prompt, estimates how complex it actually is, and routes it to one of three pre-configured `RemoteLlm` tiers, instead of always paying for the biggest model and the largest response budget regardless of what was actually asked.

> **Requirements:** a real LLM provider API key. The server, routing, and the actual outbound request to the provider are all verified end to end with `melodium run` and real `curl` requests; only the provider's own reply needs a real key to observe.

## What it does

```
melodium run Compo.toml --api_key sk-...

curl -X POST http://127.0.0.1:8080/chat -d "What year is it?"
curl -X POST http://127.0.0.1:8080/chat -d "Can you summarize the main differences between REST and GraphQL APIs?"
curl -X POST http://127.0.0.1:8080/chat -d "Explain, step by step, how a hash map resizes, and compare it to a B-tree's rebalancing cost."
```

Verified with `melodium run` against the real server, HTTP requests included: the first request (short, factual) is routed to the `economy` tier; the second (moderate length, one complexity signal) to `standard`; the third (three distinct complexity signals: "explain", "step by step", "compare") to `premium`. The server log prints the router's decision and its reasoning for every request:

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

### Reference

- **`encoding`**: [decode](https://doc.melodium.tech/latest/en/encoding/decode.html), [encode](https://doc.melodium.tech/latest/en/encoding/encode.html)
- **`http`**: [HttpServer](https://doc.melodium.tech/latest/en/http/server/HttpServer.html), [start](https://doc.melodium.tech/latest/en/http/server/start.html), [connection](https://doc.melodium.tech/latest/en/http/server/connection.html), [|post](https://doc.melodium.tech/latest/en/http/method/|post.html), [|ok](https://doc.melodium.tech/latest/en/http/status/|ok.html), [HttpStatus](https://doc.melodium.tech/latest/en/http/status/HttpStatus.html)
- **`javascript`**: [JavaScriptEngine](https://doc.melodium.tech/latest/en/javascript/JavaScriptEngine.html), [process](https://doc.melodium.tech/latest/en/javascript/process.html)
- **`json`**: [Json](https://doc.melodium.tech/latest/en/json/Json.html), [toJson](https://doc.melodium.tech/latest/en/json/toJson.html), [fromString](https://doc.melodium.tech/latest/en/json/value/fromString.html), [|null](https://doc.melodium.tech/latest/en/json/value/|null.html)
- **`ml`**: [RemoteLlm](https://doc.melodium.tech/latest/en/ml/remote/llm/RemoteLlm.html), [llmStream](https://doc.melodium.tech/latest/en/ml/remote/llm/stream.html)
- **`net`**: [|localhost_ipv4](https://doc.melodium.tech/latest/en/net/ip/|localhost_ipv4.html), [|from_ipv4](https://doc.melodium.tech/latest/en/net/ip/|from_ipv4.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logInfos](https://doc.melodium.tech/latest/en/std/engine/log/logInfos.html), [logErrors](https://doc.melodium.tech/latest/en/std/engine/log/logErrors.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [stream](https://doc.melodium.tech/latest/en/std/flow/stream.html), [trigger](https://doc.melodium.tech/latest/en/std/flow/trigger.html), [filterBlock](https://doc.melodium.tech/latest/en/std/flow/filterBlock.html), [merge](https://doc.melodium.tech/latest/en/std/flow/merge.html), [equalTo](https://doc.melodium.tech/latest/en/std/ops/block/equalTo.html), [StringMap](https://doc.melodium.tech/latest/en/std/data/string_map/StringMap.html), [|map](https://doc.melodium.tech/latest/en/std/data/string_map/|map.html), [unwrapOr](https://doc.melodium.tech/latest/en/std/ops/option/unwrapOr.html), [|wrap](https://doc.melodium.tech/latest/en/std/ops/option/|wrap.html), [toString](https://doc.melodium.tech/latest/en/std/conv/toString.html), [tryToString](https://doc.melodium.tech/latest/en/std/conv/tryToString.html)

## Runtime behaviour

1. Mélodium's `RemoteLlm` sets `model` and `max_tokens` once per model instance, not per request, so "optimise the token budget for this request" cannot mean "compute an arbitrary number every time"; it means "pick the right one of a few pre-defined `(model, budget)` tiers": `economyLlm` (small, fast model, 200-token budget), `standardLlm` (600 tokens), `premiumLlm` (most capable model, 1500 tokens).
2. The request body is collapsed to a single `Block<string>` (the `trigger.last` idiom from example 03), wrapped as `Json` with `fromString`, and passed to the JS `decide()` function, which returns a JSON object with the chosen `tier`, a word/token estimate, and a human-readable `reason`. That whole object is logged server-side for every request.
3. A second, tiny JS call, `pickTier(decision)`, projects just the `tier` field back out; there is still no field-by-field access into a parsed `Json` value outside JavaScript (see 08_javascript_transform), so chaining a second `process` call on the first call's own output is the way to read one field out of a result you already computed in JS.
4. The prompt is routed with three `equalTo` + `filterBlock` gates, one per tier (the block-level counterparts of `filter`, used everywhere else in this tutorial on streams). Exactly one gate's `accepted` output actually carries the prompt; the other two close empty. Because an LLM `stream` treatment fed an empty prompt stream never calls the provider at all, the two tiers *not* chosen cost nothing, not even a request.
5. The three (mostly empty) token streams are combined with two `merge`s into one; since only one branch ever produced anything, the merged stream is just that branch's output.
6. `temperature` is set explicitly to `|wrap<f32>(1.0)`. Left as `_`, `RemoteLlm`'s `temperature` (a Rust-declared model parameter) does not omit the value the way a `.mel`-declared `Option<T>` would: it is sent as `0`, which Anthropic's newest models reject outright (`` `temperature` is deprecated for this model``). `1.0` is the one value they still accept. `top_p`, left as `_`, is correctly omitted.

### Key Mélodium patterns used

- **A model's constant parameters are the unit of "configuration", not the request.** When something needs to vary per request but the model only exposes it as a fixed parameter, the answer is several model instances, not a dynamic parameter.
- **JS as a two-step pipeline**: one `process` call to compute a whole decision object, a second `process` call (same engine, same compiled functions) to project one field out of it. Chaining calls like this is cheaper than re-deriving the projection outside JS, which usually is not possible anyway (see point 3 above).
- **`equalTo` + `filterBlock`**: the block-level shape of the `exact`/`filter` combination used on streams throughout this tutorial, for routing a single request rather than filtering a sequence.
- **An empty prompt stream costs nothing.** Feeding all three tiers and letting the unchosen ones receive zero items is simpler than building a true N-way dynamic dispatch, and is free: an LLM treatment that never receives a prompt never calls the provider.
- **Verify logic in isolation before wiring it into something you can't easily test** (here, real API calls): the same principle as `melodium run` over `melodium check` elsewhere in this tutorial, applied to a piece of business logic rather than a library treatment.
- **`_` on a Rust-declared model parameter is not the same thing as `_` on a `.mel`-declared one.** The latter genuinely omits an `Option<T>`; the former can silently send a zero value instead. Prefer an explicit `|wrap<T>(value)` over `_` for a Rust-declared model's optional numeric parameters until this is fixed at the source.

Back to the [examples index](../../README.md).
