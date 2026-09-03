# Mélodium examples

Two tracks:

- **[`tutorial/`](tutorial/)**: a guided path through Mélodium's core concepts, one at a time, in increasing order of difficulty. Start at [`01_hello_melodium`](tutorial/01_hello_melodium/) if you are new to the language.
- **[`showcase/`](showcase/)**: a handful of larger, more convincing demos that combine several capabilities at once (remote LLMs, speech, distributed compute, CI/CD) without stopping to explain every concept along the way. Read these once the tutorial track feels comfortable.

Every example is a self-contained Mélodium project (`Compo.toml` + `.mel` files) with its own `README.md` explaining what it does and why it is built the way it is. Run any of them with:

```
melodium run <path-to-example>/Compo.toml [arguments...]
```

Examples with more than one entrypoint are run with `melodium run Compo.toml <entrypoint> [arguments...]`; see each example's `README.md` for its exact invocation.

Most tutorial examples run entirely locally and were verified end to end while writing them (`melodium check` **and** an actual `melodium run`, inspecting the real output: see each README's *Runtime behaviour* section for specifics, including a few real bugs that were found and fixed this way). A few examples (anything needing a real database, a paid API key, a second Mélodium engine, or cloud infrastructure) were verified with `melodium check` only; their README says so explicitly.

## Capability inventory

The table below is a map of what Mélodium (and its standard library packages) can do, and which example(s) demonstrate it. It is not exhaustive: see the [Mélodium reference documentation](https://doc.melodium.tech/latest/en/) for the full standard library.

| Domain | Package(s) | Demonstrated in |
|---|---|---|
| Treatments, `Block`/`Stream`, connections, functions, `startup()` | language | [tutorial/01](tutorial/01_hello_melodium/) |
| Generic treatments with trait bounds | language | [tutorial/02](tutorial/02_flow_and_generics/) |
| Flow control: `filter`, `merge`, `generate`, `count`, `fill`, `trigger` | `std/flow` | [tutorial/02](tutorial/02_flow_and_generics/), [tutorial/03](tutorial/03_text_and_files/) |
| Arithmetic & comparison | `std/ops`, `std/ops/num` | [tutorial/02](tutorial/02_flow_and_generics/) |
| Text processing & regex | `std/text/compose`, `std/text/compare`, `regex` | [tutorial/03](tutorial/03_text_and_files/) |
| File I/O | `fs/local`, `fs/text` | [tutorial/03](tutorial/03_text_and_files/), [tutorial/08](tutorial/08_javascript_transform/), [tutorial/09](tutorial/09_process_pipeline/) |
| Structured data: `Map`, `StringMap` | `std/data/map`, `std/data/string_map` | [tutorial/03](tutorial/03_text_and_files/), [tutorial/06](tutorial/06_http_server_api/), [tutorial/07](tutorial/07_sql_crud_api/) |
| JSON parsing, validation, construction | `json` | [tutorial/04](tutorial/04_json_toolkit/) |
| HTTP client | `http/client` | [tutorial/05](tutorial/05_http_client/) |
| HTTP server, routing, `@HttpRequest` context | `http/server` | [tutorial/06](tutorial/06_http_server_api/) |
| SQL: connection pools, `fetch`, `execute` | `sql` | [tutorial/07](tutorial/07_sql_crud_api/) |
| Embedded JavaScript execution | `javascript` | [tutorial/08](tutorial/08_javascript_transform/), [showcase/smart_llm_router](showcase/smart_llm_router/) |
| External process execution | `process` | [tutorial/09](tutorial/09_process_pipeline/) |
| Distributed dataflow: running a treatment on another engine | `distrib` | [tutorial/10](tutorial/10_distributed_computation/), [showcase/distributed_llm_cluster](showcase/distributed_llm_cluster/) |
| On-demand cloud workers | `work` | [tutorial/10](tutorial/10_distributed_computation/), [showcase/distributed_llm_cluster](showcase/distributed_llm_cluster/), [showcase/ci_pipeline](showcase/ci_pipeline/) |
| Remote LLM chat (streaming & non-streaming) | `ml/remote/llm` | [showcase/ai_voice_assistant](showcase/ai_voice_assistant/), [showcase/distributed_llm_cluster](showcase/distributed_llm_cluster/), [showcase/smart_llm_router](showcase/smart_llm_router/) |
| Local speech-to-text (Whisper) & remote text-to-speech | `ml/models/whisper`, `ml/remote/tts`, `record/audio` | [showcase/ai_voice_assistant](showcase/ai_voice_assistant/) |
| HuggingFace Hub model download | `ml/repos/hf` | [showcase/ai_voice_assistant](showcase/ai_voice_assistant/) |
| CI/CD pipelines, containerised steps, service containers | `cicd` | [showcase/ci_pipeline](showcase/ci_pipeline/) |

Packages with no dedicated example: `audio` (decode/encode/resample audio formats, used indirectly wherever `record/audio` is), `net` (IP address handling, used throughout as plumbing for `http`/`distrib`), `encoding` (byte↔string conversion, used throughout).

## A note on verification

While writing this set, several real bugs were found by actually *running* examples instead of only type-checking them, notably: `std/flow::count` starts at 0 despite its own documentation; a decoded byte stream (HTTP body, subprocess stdout) can end with a spurious empty chunk; and an HTTP server route must be triggered from `connection.started`, not from a stream derived from the (possibly empty) request body: the last one silently breaks every `GET` route built the naive way. Each of these is called out in the specific example's README where it was found. If you hit something that looks wrong while adapting these examples, try `melodium run` with real logging before assuming your graph is correct: `melodium check` only verifies types, not runtime behaviour.
