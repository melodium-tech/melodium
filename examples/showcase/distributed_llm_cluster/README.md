# Distributed LLM Cluster: showcase

Not a tutorial step: builds directly on [10_distributed_computation](../../tutorial/10_distributed_computation/)'s `distrib` primitives, adding `work/distant` to provision the remote engine on demand from Mélodium Services, instead of pointing at a `melodium dist` node started by hand.

> **Requirements:** a Mélodium Services API token and an LLM provider API key. Set `MELODIUM_API_TOKEN` in the environment and run with `--api-report`; see Cadence.CI to obtain a token and follow execution. Verified end to end against real infrastructure: a real worker provisioned via `work/distant`, and a real LLM response returned through it.

## What it does

An HTTP server accepts `POST /chat` with a plain-text prompt and streams the LLM's response back. The `ml` package (and whatever API credentials or compute it needs) only has to be available on the *worker* Mélodium provisions on demand, never on the front-end process itself.

```
export MELODIUM_API_TOKEN="my-melodium-services-token"
melodium run --api-report Compo.toml \
  --llm_api_key sk-... \
  --port 8080

curl -X POST http://127.0.0.1:8080/chat \
     -d "Explain the Mélodium dataflow model in one sentence."
```

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `runner` | `DistantEngine` | Requests a worker from Mélodium Services |
| `distributor` | `DistributionEngine` | Names the remote treatment (`inferText`) to run there |
| `httpServer` | `HttpServer` | Front-end HTTP listener |
| `Assistant` (`llm`) | `RemoteLlm` | Instantiated *on the worker*, inside `inferText` |

### Data flow

```
distant (provision worker) ──▶ distrib start (connect) ──▶ HTTP server starts
                                                                  │
                                     POST /chat ──▶ dispatchInfer ──send/recv──▶ inferText (remote) ──▶ chat (LLM)
```

### Reference

- **`distrib`**: [DistributionEngine](https://doc.melodium.tech/latest/en/distrib/DistributionEngine.html), [distribStart](https://doc.melodium.tech/latest/en/distrib/start.html), [distribute](https://doc.melodium.tech/latest/en/distrib/distribute.html), [sendStream](https://doc.melodium.tech/latest/en/distrib/sendStream.html), [recvStream](https://doc.melodium.tech/latest/en/distrib/recvStream.html)
- **`encoding`**: [decode](https://doc.melodium.tech/latest/en/encoding/decode.html), [encode](https://doc.melodium.tech/latest/en/encoding/encode.html)
- **`http`**: [HttpServer](https://doc.melodium.tech/latest/en/http/server/HttpServer.html), [start](https://doc.melodium.tech/latest/en/http/server/start.html), [connection](https://doc.melodium.tech/latest/en/http/server/connection.html), [|post](https://doc.melodium.tech/latest/en/http/method/|post.html), [|ok](https://doc.melodium.tech/latest/en/http/status/|ok.html), [HttpStatus](https://doc.melodium.tech/latest/en/http/status/HttpStatus.html)
- **`ml`**: [RemoteLlm](https://doc.melodium.tech/latest/en/ml/remote/llm/RemoteLlm.html), [chat](https://doc.melodium.tech/latest/en/ml/remote/llm/chat.html)
- **`net`**: [|localhost_ipv4](https://doc.melodium.tech/latest/en/net/ip/|localhost_ipv4.html), [|from_ipv4](https://doc.melodium.tech/latest/en/net/ip/|from_ipv4.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logError](https://doc.melodium.tech/latest/en/std/engine/log/logError.html), [logErrorMessage](https://doc.melodium.tech/latest/en/std/engine/log/logErrorMessage.html), [logErrors](https://doc.melodium.tech/latest/en/std/engine/log/logErrors.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [trigger](https://doc.melodium.tech/latest/en/std/flow/trigger.html), [StringMap](https://doc.melodium.tech/latest/en/std/data/string_map/StringMap.html), [|map](https://doc.melodium.tech/latest/en/std/data/string_map/|map.html), [|dataMap](https://doc.melodium.tech/latest/en/std/data/map/|map.html), [|dataEntry](https://doc.melodium.tech/latest/en/std/data/map/|entry.html), [|wrap](https://doc.melodium.tech/latest/en/std/ops/option/|wrap.html)
- **`work`**: [DistantEngine](https://doc.melodium.tech/latest/en/work/distant/DistantEngine.html), [distant](https://doc.melodium.tech/latest/en/work/distant/distant.html)

## Notable choices

- **Scaling to an actual cluster** means provisioning several workers, each with its own `distant` + `DistributionEngine` pair, and load-balancing requests across them: the same `dispatchInfer` shape, repeated N times. This example provisions one worker for clarity; the fan-out itself is not implemented here.
- **`connection.started`**, as established in [06_http_server_api](../../tutorial/06_http_server_api/), gates the HTTP response, independent of whether or when the distributed round-trip completes.
- **Credentials travel with the distribution, not the request**: the LLM API key is passed once as a `params` entry to `distrib::start` (`|dataMap([|dataEntry<string>(...), ...])`, note the explicit `<string>`: `std/data/map::|entry` is generic since `Map`, unlike `StringMap`, holds values of any type) and read by `inferText` on the worker side, not re-sent with every prompt.
- **The remote treatment is just a treatment**: `inferText` reads bytes, decodes, calls `chat`, encodes, writes bytes; nothing about it is aware it is running on a different machine than the code that calls it via `dispatchInfer`.

Back to the [examples index](../../README.md).
