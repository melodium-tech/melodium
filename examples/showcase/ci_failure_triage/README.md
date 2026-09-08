# CI Failure Triage

Two CI steps run on provisioned containers, and each step's raw output is deterministically classified by a small JavaScript function before anything else happens. Only the step the classifier actually flagged as failed is handed to a remote LLM for a plain-language diagnosis and a suggested fix; the step the classifier considers fine costs nothing beyond the classification itself, no LLM request is made for it at all.

> **Requirements:** a Mélodium Services API token and an LLM provider API key. Set `MELODIUM_API_TOKEN` in the environment and run with `--api-report`; see [Cadence.CI](https://cadence.ci/) to obtain a token and follow execution.

## What it does

```
export MELODIUM_API_TOKEN="my-melodium-services-token"
melodium run --api-report Compo.toml --api_key sk-...
```

Runs two steps on the stock `python:3.13-slim` image, nothing to clone or install:

- **`unit_tests`**: three trivial, always-true assertions. Always passes.
- **`integration_check`**: reads a config dictionary by a misspelled key (`'tiemout'` instead of `'timeout'`). Always fails with a real `KeyError`.

Both are deliberately deterministic so this example runs the same way every time, independent of any external repository's current state. `ci_failure_report.md` ends up looking like this:

```markdown
## unit_tests

Status: **passed**
Classification: `{"category":"none","status":"passed","summary":"no known failure signature found in the step output"}`

No issues detected by the deterministic classifier; no AI analysis was requested for this step.

## integration_check

Status: **failed**
Classification: `{"category":"key_error","status":"failed","summary":"A dictionary key does not exist: tiemout"}`

## Root Cause

The script contains a typo in the dictionary key lookup: `config['tiemout']` instead of `config['timeout']`. Python raises a `KeyError` because `'tiemout'` does not exist as a key in the `config` dictionary. The first `print` succeeds (retries prints correctly), but execution halts on the second `print` when it tries to access the misspelled key.

## Suggested Fix

Correct the typo in the key name from `'tiemout'` to `'timeout'`:

​```python
# Before (broken)
print('effective timeout:', config['tiemout'])

# After (fixed)
print('effective timeout:', config['timeout'])
​```

This is a one-character transposition (`ie` → `ei`) and is the only change needed.
```

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `dispatcher` | `CicdDispatchEngine` | Spawns each step's container |
| `classifier` | `JavaScriptEngine` | Holds `classify()` and `statusOf()`, compiled once at startup |
| `analyst` | `RemoteLlm` | Diagnoses and suggests a fix for a step the classifier flagged as failed |

### Data flow

```
step ──▶ capture log ──▶ classify() (JS) ──▶ log full classification
                      │                   └─▶ statusOf() (JS) ──▶ status
                      │
                      ├──▶ [status == failed] ──▶ analyst.chat  ──┐
                      └──▶ [status == passed] ──▶ fixed message ──┴─▶ merge ──▶ report section
```

Run twice (`unit_tests`, `integration_check`), and the two sections are combined into one file.

### Reference

- **`cicd`**: [CicdDispatchEngine](https://doc.melodium.tech/latest/en/cicd/runners/CicdDispatchEngine.html), [simpleStep](https://doc.melodium.tech/latest/en/cicd/naive/simpleStep.html)
- **`encoding`**: [decode](https://doc.melodium.tech/latest/en/encoding/decode.html)
- **`fs`**: [writeTextLocal](https://doc.melodium.tech/latest/en/fs/local/writeTextLocal.html)
- **`javascript`**: [JavaScriptEngine](https://doc.melodium.tech/latest/en/javascript/JavaScriptEngine.html), [process](https://doc.melodium.tech/latest/en/javascript/process.html)
- **`json`**: [Json](https://doc.melodium.tech/latest/en/json/Json.html), [fromString](https://doc.melodium.tech/latest/en/json/value/fromString.html), [|null](https://doc.melodium.tech/latest/en/json/value/|null.html)
- **`ml`**: [RemoteLlm](https://doc.melodium.tech/latest/en/ml/remote/llm/RemoteLlm.html), [llmChat](https://doc.melodium.tech/latest/en/ml/remote/llm/chat.html)
- **`process`**: [Command](https://doc.melodium.tech/latest/en/process/command/Command.html), [|command](https://doc.melodium.tech/latest/en/process/command/|command.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logInfos](https://doc.melodium.tech/latest/en/std/engine/log/logInfos.html), [logErrorMessage](https://doc.melodium.tech/latest/en/std/engine/log/logErrorMessage.html), [logErrors](https://doc.melodium.tech/latest/en/std/engine/log/logErrors.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [stream](https://doc.melodium.tech/latest/en/std/flow/stream.html), [trigger](https://doc.melodium.tech/latest/en/std/flow/trigger.html), [filterBlock](https://doc.melodium.tech/latest/en/std/flow/filterBlock.html), [merge](https://doc.melodium.tech/latest/en/std/flow/merge.html), [equalTo](https://doc.melodium.tech/latest/en/std/ops/block/equalTo.html), [StringMap](https://doc.melodium.tech/latest/en/std/data/string_map/StringMap.html), [blockEntry](https://doc.melodium.tech/latest/en/std/data/string_map/block/entry.html), [blockInsert](https://doc.melodium.tech/latest/en/std/data/string_map/block/insert.html), [unwrapOr](https://doc.melodium.tech/latest/en/std/ops/option/unwrapOr.html), [|wrap](https://doc.melodium.tech/latest/en/std/ops/option/|wrap.html), [toString](https://doc.melodium.tech/latest/en/std/conv/toString.html), [tryToString](https://doc.melodium.tech/latest/en/std/conv/tryToString.html), [format](https://doc.melodium.tech/latest/en/std/text/compose/format.html)

## Runtime behaviour

1. `simpleStep`'s `commands` runs each entry as a direct exec, never through a shell: `>`, `2>&1`, `;`, `&&`, and `cd` are only interpreted when the command itself is `sh -c "..."`, which is why each step's command is `|command("sh", ["-c", "<the real command>"])` rather than something built with `|raw_commands`.
2. `readFile` (what actually produces `data`) does not look at the step's exit code at all: only a genuine executor-level failure (the container could not be spawned) skips it. A step whose real command exits non-zero still gets its `out_file` streamed back. Each step's real command still ends with `; true`, but that is about the step's own `success`/`error` signal being uninteresting here (this graph makes its own judgement from the captured text), not about getting `data` back at all.
3. The captured log is decoded, collapsed to a `Block<string>` (the `trigger.last` idiom from [03_text_and_files](../../tutorial/03_text_and_files/)), wrapped as `Json`, and passed to the JS `classify()` function, which matches it against a short list of known failure signatures (`KeyError`, `TypeError`, a missing module, a `SyntaxError`, a failed `assert`, or a generic unhandled exception) and returns a `status`/`category`/`summary` object. That whole object is logged for every step.
4. A second, tiny JS call, `statusOf(decision)`, projects just the `status` field back out, the same two-step JS pipeline as `showcase/smart_llm_router`.
5. Two `equalTo` + `filterBlock` gates route on that status: `failed` gates the raw log into the LLM's `prompt`; `passed` gates a fixed, free message instead. Exactly one gate ever carries anything, so exactly one of "call the LLM" or "say nothing needed" ever happens, and the merge of the two (mostly empty) branches is just whichever one fired.
6. Each step's section (name, status, the full classification, and the analysis or the fixed message) is assembled with the `blockEntry`/`blockInsert`/`format` pattern from [03_text_and_files](../../tutorial/03_text_and_files/) and [04_json_toolkit](../../tutorial/04_json_toolkit/), and the two steps' sections are combined the same way once more at the top level before being written to `ci_failure_report.md`.

### Key Mélodium patterns used

- **`commands` is not a shell, no matter how shell-like the strings look.** `|raw_commands` tokenises a string; it does not interpret `${VAR}`, `&&`, `cd`, or redirection. Anything that needs those needs an explicit `|command("sh", ["-c", "..."])`.
- **Classify before you spend a request.** The same idea as `showcase/smart_llm_router`'s tiers, applied to a binary decision this time: call the LLM only when a fast, free, deterministic check says it is worth it.
- **`chat` vs `stream`**: `ml/remote/llm::chat` returns one complete response per prompt rather than tokens as they arrive; the right choice when the result is going into a report rather than out over a live connection.
- **A generic treatment reused across instances**: `ciStepAnalysis` is written once and instantiated twice (`unitTests`, `integrationCheck`), each with its own step name, image, and commands.

Back to the [examples index](../../README.md).
