# Distributed LLM Inference

An HTTP server that accepts plain-text prompts, forwards them to an LLM running on a provisioned cloud runner, and streams the response back to the caller. The LLM (`ml` package) runs entirely on the remote node: the front-end machine requires no GPU or ML dependencies.

## What it does

- Provisions a cloud ML runner via the Mélodium Services API on startup.
- Connects a `DistributionEngine` to the runner, passing the `openai_key` to the remote treatment at connection time.
- Starts an HTTP server once the runner is connected.
- Every `POST /chat` request body (plain text) is the prompt.
- The prompt is sent to the remote `inferText` treatment, which calls the OpenAI API via `RemoteLlm`.
- Response tokens stream back and are forwarded as the HTTP response body.

```
melodium run Compo.toml -- \
  --api_token  "my-api-token" \
  --openai_key sk-... \
  --port 8080

curl -X POST http://127.0.0.1:8080/chat \
     -H "Content-Type: text/plain" \
     -d "Explain the Mélodium dataflow model in one sentence."
```

## How it is built

### Models

| Model | Type | Purpose |
|---|---|---|
| `runner` | `DistantEngine` | Provisions cloud ML runner via Mélodium Services API |
| `distributor` | `DistributionEngine` | Routes work to `distributed_llm_inference/main::inferText` |
| `httpServer` | `HttpServer` | HTTP listener on localhost |
| `Assistant` | `RemoteLlm` | GPT-4o-mini via OpenAI API, instantiated on the remote runner |

### Treatments

**`server`** is the entry point. It provisions the runner, connects the distributor with `openai_key` in the params map, starts the HTTP server, and wires connections to `dispatchInfer`.

**`dispatchInfer[distributor]`** is the per-request bridge:
- `trigger<byte>()` fires on the first prompt byte.
- `distribute` allocates a distribution slot.
- `sendStream<byte>(name="prompt")` tunnels the raw prompt bytes to the remote treatment.
- `recvStream<byte>(name="response")` collects the encoded response bytes.

**`inferText(const openai_key: string)`** executes remotely:
1. `decode` converts prompt bytes to a UTF-8 string.
2. `chat[llm]` calls the remote LLM (GPT-4o-mini) via the `Assistant` model.
3. `encode` converts response tokens back to bytes.
4. LLM errors are logged via `logErrors(label="llm")`.

The `openai_key` parameter of `inferText` is `const`: it is set once when the distribution engine starts (`start(params=|map([|entry<string>("openai_key", openai_key)]))`) and shared across all invocations.

## Distribution architecture

```mermaid
graph LR
    subgraph local["Local machine\n(no ML dependencies needed)"]
        HTTP["HTTP Server\n(port 8080)"]
        DISPATCH["dispatchInfer\n(distribute + send/recv)"]
        DIST["DistributionEngine\n(openai_key passed at start)"]
        DISTANT["DistantEngine\n(API provisioning)"]
    end

    subgraph cloud["Cloud ML Runner (provisioned on demand)"]
        INFER["inferText treatment\n(const openai_key)"]
        LLM["Assistant model\n(RemoteLlm → GPT-4o-mini)"]
        INFER -->|chat| LLM
        LLM -->|OpenAI API| OPENAI[("OpenAI\nAPI")]
    end

    USER["HTTP client"] -->|POST /chat\n(prompt bytes)| HTTP
    HTTP -->|bytes| DISPATCH
    DISPATCH -->|sendStream 'prompt'| INFER
    INFER -->|recvStream 'response'| DISPATCH
    DISPATCH -->|response bytes| HTTP
    HTTP -->|streaming response| USER

    DISTANT -->|provisions runner| cloud
    DIST -->|openai_key + distribution_id| INFER
```

## Runtime behaviour

1. **Runner provisioning**: `DistantEngine` contacts `https://api.melodium.tech/0.1` and provisions a runner (512 MB RAM, 1 CPU, 512 MB storage). `logStart` fires at startup; `distantErrLog` / `distantFailLog` catch any provisioning errors.

2. **Distribution start**: `provisionRunner.access` flows to `distribStart.access`. `distribStart` is called with `params=|map([|entry<string>("openai_key", openai_key)])`, which passes the OpenAI key to the remote `inferText` treatment as its `const openai_key` parameter. This is the only mechanism to pass parameters to a remote treatment; `distribute` itself has no params in the Mélodium DSL.

3. **HTTP server start**: Only after `distribStart.ready` fires do both `startHttp` and `logServerReady` execute. Requests cannot arrive before the remote worker is ready.

4. **Per-request inference**:
   - A new track is created for each `POST /chat`.
   - `bodyTrigger` gates status 200 and headers.
   - `dispatchInfer` calls `distribute` for a `distribution_id`, sends prompt bytes via `sendStream<byte>(name="prompt")`, and receives response bytes via `recvStream<byte>(name="response")`.
   - On the runner, `inferText` decodes the prompt, calls `chat[llm]` (OpenAI API, GPT-4o-mini), encodes each token to bytes, and streams them back.
   - Response bytes arrive at `dispatchInfer.response` and flow directly into `connection.data`: the HTTP client sees tokens arrive progressively.

5. **Concurrency**: Multiple simultaneous HTTP requests each get their own track and their own `distribution_id`. The distribution engine multiplexes them all over the same channel to the runner.

### Key Mélodium patterns used

- **`start(params=|map([|entry<string>("key", value)]))`**: the only way to pass `const` parameters to a remote treatment; these are set once at distribution engine start time, not per-request.
- **`const` remote parameter**: `inferText` uses `const openai_key` because it is set once by `start`'s params and shared across all invocations of that treatment on the runner. A `var` parameter would require per-invocation data, which is handled by `sendStream` / `recvStream`.
- **Front-end / back-end separation**: the `ml` package (and its API call logic) lives entirely on the runner. The front-end machine only needs `http`, `distrib`, and `work` packages.
- **`|wrap<string>(...)`**: `DistantEngine`'s `api_url` and `api_token` fields are `Option<string>`; `|wrap<string>` promotes a plain string into `Some(string)`.
- **Token-level HTTP streaming**: because `inferText.response` is `Stream<byte>` and flows directly to `connection.data`, the HTTP client receives each LLM token as it is generated on the remote runner, with minimal buffering.
