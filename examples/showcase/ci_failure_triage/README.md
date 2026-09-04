# CI Failure Triage: showcase

Not a tutorial step: two CI steps run on provisioned containers, and each step's raw output is deterministically classified by a small JavaScript function before anything else happens. Only the step the classifier actually flagged as failed is handed to a remote LLM for a plain-language diagnosis and a suggested fix; the step the classifier considers fine costs nothing beyond the classification itself, no LLM request is made for it at all.

> **Requirements:** a Mélodium Services API token and an LLM provider API key. Set `MELODIUM_API_TOKEN` in the environment and run with `--api-report`; see Cadence.CI to obtain a token and follow execution. This example is type-checked with `melodium check`; running it against live infrastructure was not part of this tutorial (no token was available in the environment that wrote it), but every assumption it makes about the two steps' output was verified locally beforehand, and its JavaScript classifier and LLM wiring were each verified in isolation with `melodium run` (see *How it was checked* below).

## What it does

```
export MELODIUM_API_TOKEN="my-melodium-services-token"
melodium run --api-report Compo.toml --api_key sk-...
```

Runs two steps on the stock `python:3.13-slim` image, nothing to clone or install:

- **`unit_tests`**: three trivial, always-true assertions. Always passes.
- **`integration_check`**: reads a config dictionary by a misspelled key (`'tiemout'` instead of `'timeout'`). Always fails with a real `KeyError`.

Both are deliberately deterministic so this example runs the same way every time, independent of any external repository's current state. For each step, the report gets one section:

```markdown
## unit_tests

Status: **passed**
Classification: `{"category":"none","status":"passed","summary":"no known failure signature found in the step output"}`

No issues detected by the deterministic classifier; no AI analysis was requested for this step.

## integration_check

Status: **failed**
Classification: `{"category":"key_error","status":"failed","summary":"A dictionary key does not exist: tiemout"}`

<the LLM's diagnosis and suggested fix>
```

written to `ci_failure_report.md`.

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `dispatcher` | `CicdDispatchEngine` | Spawns each step's container |
| `classifier` | `JavaScriptEngine` | Holds `classify()` and `statusOf()`, compiled once at startup |
| `analyst` | `RemoteLlm` | Diagnoses and suggests a fix for a step the classifier flagged as failed |

### Data flow

```
step ──▶ capture log (always, see below) ──▶ classify() (JS) ──▶ log full classification
                                          │                   └─▶ statusOf() (JS) ──▶ status
                                          │
                                          ├──▶ [status == failed] ──▶ analyst.chat  ──┐
                                          └──▶ [status == passed] ──▶ fixed message ──┴─▶ merge ──▶ report section
```

Run twice (`unit_tests`, `integration_check`), and the two sections are combined into one file.

## Runtime behaviour

1. `simpleStep`'s `data` output only ever carries `out_file`'s bytes when every command in the step exits `0`; a step whose real command fails would leave `data` empty, with nothing to classify. Each step's `commands` works around this deliberately: the real command redirects its own output to `/mnt/data/log.txt` and ends with `; true`, so the container always exits `0` and the log is always streamed back. Whether the step actually succeeded is then something `ciStepAnalysis` decides for itself from the captured text, not something it trusts the container's own exit code for.
2. The captured log is decoded, collapsed to a `Block<string>` (the `trigger.last` idiom from example 03), wrapped as `Json`, and passed to the JS `classify()` function, which matches it against a short list of known failure signatures (`KeyError`, `TypeError`, a missing module, a `SyntaxError`, a failed `assert`, or a generic unhandled exception) and returns a `status`/`category`/`summary` object. That whole object is logged for every step.
3. A second, tiny JS call, `statusOf(decision)`, projects just the `status` field back out, the same two-step JS pipeline as `showcase/smart_llm_router`.
4. Two `equalTo` + `filterBlock` gates route on that status: `failed` gates the raw log into the LLM's `prompt`; `passed` gates a fixed, free message instead. Exactly one gate ever carries anything, so exactly one of "call the LLM" or "say nothing needed" ever happens, and the merge of the two (mostly empty) branches is just whichever one fired.
5. Each step's section (name, status, the full classification, and the analysis or the fixed message) is assembled with the same `blockEntry`/`blockInsert`/`format` pattern used for reports in examples 03 and 04, and the two steps' sections are combined the same way once more at the top level before being written to `ci_failure_report.md`.

### How it was checked

No Mélodium Services token was available while writing this, so the CI dispatch itself was not run live. Three things were, independently:

- **Both Python one-liners, run locally** with a plain `python3` interpreter (no Mélodium involved): `unit_tests`'s command prints `all checks passed` and exits `0`; `integration_check`'s command raises a real `KeyError: 'tiemout'` with a genuine traceback, exactly as the classifier expects.
- **The JS classifier, run in isolation** with `melodium run`, fed the exact log text captured from those two real local runs: the passing log classifies as `{"category":"none","status":"passed",...}`, the failing log as `{"category":"key_error","status":"failed","summary":"A dictionary key does not exist: tiemout"}`, correctly picking the typo'd key out of the traceback.
- **The LLM `chat` wiring, run in isolation** against the real Anthropic endpoint with a deliberately invalid key: `diag.prompt` reached `https://api.anthropic.com/v1/messages` and `diag.error` correctly received a real `401 Unauthorized`, confirming the `prompt`/`response`/`error` ports are wired correctly (this is the same `chat` treatment, non-streaming this time rather than `stream` as in `smart_llm_router`, since a triage report wants one complete diagnosis rather than tokens arriving live).
- With no real infrastructure reachable, the whole program was also run once end to end: it does not crash or hang, and correctly logs a dispatch failure for both steps instead of producing a malformed report.

### Key Mélodium patterns used

- **Work around a platform constraint explicitly, rather than silently**: `simpleStep` only streams `out_file` back on success, so getting a failing step's output back at all means making the container's own exit code lie (`; true`) and doing the real success/failure judgement afterward, in the graph. That trade-off is stated plainly in the code and here, not hidden.
- **Classify before you spend a request.** The same idea as `showcase/smart_llm_router`'s tiers, applied to a binary decision this time: call the LLM only when a fast, free, deterministic check says it is worth it.
- **`chat` vs `stream`**: `ml/remote/llm::chat` returns one complete response per prompt rather than tokens as they arrive; the right choice when the result is going into a report rather than out over a live connection.
- **A generic treatment reused across instances**: `ciStepAnalysis` is written once and instantiated twice (`unitTests`, `integrationCheck`), each with its own step name, image, and commands, exactly the pattern from example 02 applied to a showcase-sized problem.
- **Verify what you can, honestly report what you can't**: the JS logic and the LLM wiring were each checked with real runs before being wired together, even though the full pipeline itself could not be run against live infrastructure here.

Back to the [examples index](../../README.md).
