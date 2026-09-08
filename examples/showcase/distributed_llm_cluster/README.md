# Distributed LLM Cluster

Builds directly on [10_distributed_computation](../../tutorial/10_distributed_computation/)'s `distrib` primitives, adding `work/distant` to provision a remote engine on demand from Mélodium Services, instead of pointing at a `melodium dist` node started by hand.

> **Requirements:** a Mélodium Services API token. Set `MELODIUM_API_TOKEN` in the environment and run with `--api-report`; see [Cadence.CI](https://cadence.ci/) to obtain a token and follow execution. No LLM provider API key is needed: inference runs on the provisioned worker itself, against a Mistral checkpoint downloaded from the HuggingFace Hub, not against a hosted third-party API.

## What it does

An HTTP server accepts `POST /chat` with a plain-text prompt and streams generated tokens back as they're produced. Mélodium Services provisions a remote worker, which downloads `mistralai/Mistral-7B-v0.1` from the HuggingFace Hub and loads it into memory with `ml/models/mistral`, once, the first time it's launched. The `ml` package, the downloaded weights, and the compute needed to run them only have to be available on the *worker*, never on the front-end process itself.

```
export MELODIUM_API_TOKEN="my-melodium-services-token"
melodium run --api-report Compo.toml --port 8080

curl -X POST http://127.0.0.1:8080/chat \
     -d "Explain the Mélodium dataflow model in one sentence."
```

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `runner` | `DistantEngine` | Requests a worker from Mélodium Services |
| `distributor` | `DistributionEngine` | Names the remote treatment (`inferText`) to run there |
| `httpServer` | `HttpServer` | Front-end HTTP listener |
| `hub` (`HfHub`), `mistral` (`Mistral`) | `HfHub`, `Mistral` | Instantiated *on the worker*, inside `inferText`; hold, respectively, the HuggingFace repository to fetch from and the loaded model weights |

### Data flow

```
distant (provision worker) → distrib start (connect) → warm-up request (waits on load) → HTTP server starts
                                                                                                │
                                                              POST /chat ──▶ dispatchInfer ──send/recv──▶ inferText (remote)
                                                                                                                 │
                                                                                          fetch (HuggingFace Hub, once)
                                                                                                                 │
                                                                                                      load (once)
                                                                                                                 │
                                                                                     release (holds prompt until loaded)
                                                                                                                 │
                                                                                    generate (per request, once loaded)
```

On the remote side, `inferText`'s own `startup()` fires once, when `distribStart` first launches the treatment there (not once per request), driving `fetch` (download weights from the Hub, cached after the first run) and `load` (memory-map the shards, start the inference worker thread) exactly once. Every prompt, `std/flow::release` holds behind `loadModel.loaded` before it reaches `decode`/`generate`: a prompt that arrives before load has finished simply waits there instead of being dropped by `generate`. Every subsequent request flows straight through, against the already-loaded model.

On the front-end, the worker gets one real warm-up request (`emit` a single-space prompt, `dispatchInfer` it, `trigger` on the first response byte) right after its distrib connection comes up. Since the worker-side `release` above means that request is held rather than dropped, `warmupDone.start` only fires once the worker has actually generated something (real inference readiness, not just a live connection). The HTTP server (`start[http_server=httpServer]()`) only starts once that warm-up has fired, so no client request is ever routed to the worker before its model is loaded. Every step, provisioning, connecting, warming up, and going live, is logged (`logInfoMessage`/`logError`/`logErrorMessage`/`logErrors`).

### Reference

- **`distrib`**: [DistributionEngine](https://doc.melodium.tech/latest/en/distrib/DistributionEngine.html), [distribStart](https://doc.melodium.tech/latest/en/distrib/start.html), [distribute](https://doc.melodium.tech/latest/en/distrib/distribute.html), [sendStream](https://doc.melodium.tech/latest/en/distrib/sendStream.html), [recvStream](https://doc.melodium.tech/latest/en/distrib/recvStream.html)
- **`encoding`**: [decode](https://doc.melodium.tech/latest/en/encoding/decode.html), [encode](https://doc.melodium.tech/latest/en/encoding/encode.html)
- **`http`**: [HttpServer](https://doc.melodium.tech/latest/en/http/server/HttpServer.html), [start](https://doc.melodium.tech/latest/en/http/server/start.html), [connection](https://doc.melodium.tech/latest/en/http/server/connection.html), [|post](https://doc.melodium.tech/latest/en/http/method/|post.html), [|ok](https://doc.melodium.tech/latest/en/http/status/|ok.html), [HttpStatus](https://doc.melodium.tech/latest/en/http/status/HttpStatus.html)
- **`ml`**: [HfHub](https://doc.melodium.tech/latest/en/ml/repos/hf/HfHub.html), [fetch](https://doc.melodium.tech/latest/en/ml/repos/hf/fetch.html), [Mistral](https://doc.melodium.tech/latest/en/ml/models/mistral/Mistral.html), [load](https://doc.melodium.tech/latest/en/ml/models/mistral/load.html), [generate](https://doc.melodium.tech/latest/en/ml/models/mistral/generate.html)
- **`net`**: [|localhost_ipv4](https://doc.melodium.tech/latest/en/net/ip/|localhost_ipv4.html), [|from_ipv4](https://doc.melodium.tech/latest/en/net/ip/|from_ipv4.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logError](https://doc.melodium.tech/latest/en/std/engine/log/logError.html), [logErrorMessage](https://doc.melodium.tech/latest/en/std/engine/log/logErrorMessage.html), [logErrors](https://doc.melodium.tech/latest/en/std/engine/log/logErrors.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [stream](https://doc.melodium.tech/latest/en/std/flow/stream.html), [trigger](https://doc.melodium.tech/latest/en/std/flow/trigger.html), [release](https://doc.melodium.tech/latest/en/std/flow/release.html), [StringMap](https://doc.melodium.tech/latest/en/std/data/string_map/StringMap.html), [|map](https://doc.melodium.tech/latest/en/std/data/string_map/|map.html), [|dataMap](https://doc.melodium.tech/latest/en/std/data/map/|map.html), [|dataEntry](https://doc.melodium.tech/latest/en/std/data/map/|entry.html)
- **`work`**: [DistantEngine](https://doc.melodium.tech/latest/en/work/distant/DistantEngine.html), [distant](https://doc.melodium.tech/latest/en/work/distant/distant.html)

## Notable choices

- **The worker actually runs the model.** It downloads real weights and runs real inference locally, not against a hosted API: `work/distant`'s resource request (16GB memory, 4 CPU, 32GB storage) is sized for that, for `mistralai/Mistral-7B-v0.1`'s fp16 weights plus engine overhead and Hub cache headroom.
- **The worker must have answered a warm-up request before the server starts serving**, not merely connected: a not-yet-loaded worker is not usable. `warmupDone.start` gates `http/server::start`; if provisioning or model load fails outright, it never fires and the server never opens, a deliberate fail-closed choice, always logged, never silent.
- **Nothing is dropped while the worker is still loading**: `inferText` holds each incoming prompt behind `loadModel.loaded` with `std/flow::release` before it reaches `generate`. Combined with the point above, this is what makes the warm-up request a reliable readiness signal instead of racing it: the same mechanism that stops `generate` from silently discarding early prompts is what the warm-up request relies on to eventually get an answer.
- **The repository to fetch is passed once, not per request**: `hf_repo_id` travels as a `params` entry to `distrib::start` (`|dataMap([|dataEntry<string>("repo_id", hf_repo_id)])`) and is read by `inferText` on the worker side when it instantiates its own `HfHub` model.
- **The remote treatment is just a treatment**: `inferText` reads bytes, decodes, generates, encodes, writes bytes; nothing about it is aware it is running on a different machine than the code that calls it via `dispatchInfer`. Its `model` preamble (`hub`, `mistral`) is what makes fetch/load a one-time cost rather than a per-request one, an ordinary consequence of Mélodium's model lifecycle, not anything special-cased for this example.

Back to the [examples index](../../README.md).
