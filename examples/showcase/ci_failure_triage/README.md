# CI Failure Triage: showcase

Not a tutorial step: two CI steps run on provisioned containers, and each step's raw output is deterministically classified by a small JavaScript function before anything else happens. Only the step the classifier actually flagged as failed is handed to a remote LLM for a plain-language diagnosis and a suggested fix; the step the classifier considers fine costs nothing beyond the classification itself, no LLM request is made for it at all.

> **Requirements:** a Mélodium Services API token and an LLM provider API key. Set `MELODIUM_API_TOKEN` in the environment and run with `--api-report`; see Cadence.CI to obtain a token and follow execution. Verified end to end against real infrastructure: real CI dispatch, real container output, real classification, a real LLM diagnosis (see *What it does* below for the actual output).

## What it does

```
export MELODIUM_API_TOKEN="my-melodium-services-token"
melodium run --api-report Compo.toml --api_key sk-...
```

Runs two steps on the stock `python:3.13-slim` image, nothing to clone or install:

- **`unit_tests`**: three trivial, always-true assertions. Always passes.
- **`integration_check`**: reads a config dictionary by a misspelled key (`'tiemout'` instead of `'timeout'`). Always fails with a real `KeyError`.

Both are deliberately deterministic so this example runs the same way every time, independent of any external repository's current state. Written to `ci_failure_report.md`, this is a real, unedited run:

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

## Runtime behaviour

1. `simpleStep`'s `commands` runs each entry as a direct exec, never through a shell: `>`, `2>&1`, `;`, `&&`, and `cd` are only interpreted when the command itself is `sh -c "..."`, which is why each step's command is `|command("sh", ["-c", "<the real command>"])` rather than something built with `|raw_commands`.
2. `readFile` (what actually produces `data`) does not look at the step's exit code at all: only a genuine executor-level failure (the container could not be spawned) skips it. A step whose real command exits non-zero still gets its `out_file` streamed back. Each step's real command still ends with `; true`, but that is about the step's own `success`/`error` signal being uninteresting here (this graph makes its own judgement from the captured text), not about getting `data` back at all.
3. The captured log is decoded, collapsed to a `Block<string>` (the `trigger.last` idiom from example 03), wrapped as `Json`, and passed to the JS `classify()` function, which matches it against a short list of known failure signatures (`KeyError`, `TypeError`, a missing module, a `SyntaxError`, a failed `assert`, or a generic unhandled exception) and returns a `status`/`category`/`summary` object. That whole object is logged for every step.
4. A second, tiny JS call, `statusOf(decision)`, projects just the `status` field back out, the same two-step JS pipeline as `showcase/smart_llm_router`.
5. Two `equalTo` + `filterBlock` gates route on that status: `failed` gates the raw log into the LLM's `prompt`; `passed` gates a fixed, free message instead. Exactly one gate ever carries anything, so exactly one of "call the LLM" or "say nothing needed" ever happens, and the merge of the two (mostly empty) branches is just whichever one fired.
6. Each step's section (name, status, the full classification, and the analysis or the fixed message) is assembled with the same `blockEntry`/`blockInsert`/`format` pattern used for reports in examples 03 and 04, and the two steps' sections are combined the same way once more at the top level before being written to `ci_failure_report.md`.

### Key Mélodium patterns used

- **`commands` is not a shell, no matter how shell-like the strings look.** `|raw_commands` tokenises a string; it does not interpret `${VAR}`, `&&`, `cd`, or redirection. Anything that needs those needs an explicit `|command("sh", ["-c", "..."])`.
- **Classify before you spend a request.** The same idea as `showcase/smart_llm_router`'s tiers, applied to a binary decision this time: call the LLM only when a fast, free, deterministic check says it is worth it.
- **`chat` vs `stream`**: `ml/remote/llm::chat` returns one complete response per prompt rather than tokens as they arrive; the right choice when the result is going into a report rather than out over a live connection.
- **A generic treatment reused across instances**: `ciStepAnalysis` is written once and instantiated twice (`unitTests`, `integrationCheck`), each with its own step name, image, and commands, exactly the pattern from example 02 applied to a showcase-sized problem.

Back to the [examples index](../../README.md).
