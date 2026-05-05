# GitHub Actions Migration Reference

This reference covers the patterns, treatments, and data types needed to migrate GitHub Actions CI workflows to Mélodium using the `cicd` package.

---

## Concept mapping

| GitHub Actions | Mélodium |
|---|---|
| Workflow file | `.mel` treatment |
| Job | Treatment wired with `simpleStep` or `localStep` |
| Step `run:` | `runAction[contexts=contexts]()` |
| Step `uses:` | Nested treatment call with its own `JavaScriptEngine` via `replicateContextsWithInputs` |
| Step `with:` | Parameters passed to the nested treatment |
| `${{ }}` expression | Evaluated by `JavaScriptEngine` via `githubStringEval` / `githubMapEval` |
| `jobs.<job>.needs` | Sequential wiring + `includeNeeds` |
| `jobs.<job>.outputs` | `githubJobResult` `result` output (a `Json` block) |
| `jobs.<job>.strategy.matrix` | Parallel treatment instantiation wired with `one<void>()` fan-in |
| `jobs.<job>.if` | `filterBlock<void>()` on the job's `trigger` |
| `jobs.<job>.timeout-minutes` | `max_duration` parameter of `simpleStep` / `setupRunner` |
| `jobs.<job>.concurrency` | `resource_group` on `CicdDispatchEngine` (not yet a direct parameter) |
| `jobs.<job>.environment` | Deployment protection rules are outside Mélodium scope |
| `jobs.<job>.permissions` | Handled by the caller passing the appropriate token |
| `on: push` / triggers | Entry-point treatment inputs |
| `secrets` / `env` | Parameters passed to treatments |
| `step.continue-on-error` | `continue_on_error` parameter of `runAction`, chain `continue` output |
| `step.timeout-minutes` | No direct equivalent per step; apply `max_duration` at runner level |
| Status reporting | `postGithubState` / `postGithubStateContext` |

---

## Package imports

```mel
use javascript::JavaScriptEngine
use cicd/services/github::prepareContexts
use cicd/services/github::runAction
use cicd/services/github::includeNeeds
use cicd/services/github::includeInputs
use cicd/services/github::postGithubState
use cicd/services/github::postGithubStateContext
use cicd/services/github::githubJobResult
use cicd/naive::simpleStep
use cicd/naive::localStep
use cicd::StepState
```

---

## The expression context engine

GitHub Actions `${{ }}` expressions are evaluated by a `JavaScriptEngine` model. Every job treatment that uses expressions must declare this model and initialise it with `prepareContexts`.

```mel
treatment myJob[dispatcher: CicdDispatchEngine](github_contexts: string, ...)
  input trigger: Block<void>
  output success: Block<void>
  output failure: Block<void>
{
    contexts: JavaScriptEngine()

    prepareContexts[contexts=contexts](github_contexts=github_contexts)

    // steps follow after prepareContexts.ready
    ...
}
```

`prepareContexts` receives the raw JSON string containing all GitHub Actions context objects. It loads into the engine:

| Variable | GitHub equivalent |
|---|---|
| `github` | `github` context |
| `env` | `env` context |
| `vars` | `vars` context |
| `job` | `job` context (initialised with `status = "success"`) |
| `steps` | `steps` context (populated step-by-step by `runAction`) |
| `runner` | `runner` context |
| `secrets` | `secrets` context |
| `strategy` | `strategy` context |
| `matrix` | `matrix` context |
| `needs` | `needs` context (populated by `includeNeeds`) |
| `inputs` | `inputs` context (populated by `includeInputs`) |

Helper functions `always()`, `success()`, `failure()`, `cancelled()` are also injected.

---

## Running a step — `runAction`

`runAction` mirrors a single `run:` step from GitHub Actions. It must be used inside a treatment that has a `JavaScriptEngine` model called `contexts`.

```mel
treatment myJob[dispatcher: CicdDispatchEngine](github_contexts: string)
  input trigger: Block<void>
  output completed: Block<void>
  output failed: Block<void>
{
    contexts: JavaScriptEngine()

    prepareContexts[contexts=contexts](github_contexts=github_contexts)

    buildStep: runAction[contexts=contexts](
        name="build",
        commands="cargo build --release",
        shell="bash"
    )

    trigger -> prepareContexts.trigger,ready -> buildStep.trigger
    buildStep.completed -> Self.completed
    buildStep.failed -> Self.failed
}
```

### `runAction` parameters

| Parameter | Type | Purpose |
|---|---|---|
| `name` | `string` | Step identifier, used in step context (`steps.<name>`) |
| `display_name` | `Option<string>` | Human-readable label shown in logs (falls back to `name`) |
| `commands` | `string` | Script content, supports `${{ }}` expressions |
| `env` | `Option<StringMap>` | Extra environment variables, supports expressions in values |
| `working_directory` | `Option<string>` | Working directory override, supports expressions |
| `shell` | `Option<string>` | Shell override: `"bash"`, `"sh"`, `"pwsh"`, `"powershell"`, `"python"`, `"cmd"` |
| `if` | `Option<string>` | Condition expression — step is skipped if it evaluates to `false` |
| `continue_on_error` | `Option<string>` | Expression; when truthy, emits `continue` instead of stopping on non-zero exit |
| `local_context` | `Option<string>` | JSON string merged into the expression context for this step |

### `runAction` outputs

| Output | Meaning |
|---|---|
| `completed` | Step exited with code `0` |
| `failed` | Step failed (executor or non-zero exit, `continue_on_error` not set) |
| `continue` | Step failed but `continue_on_error` was truthy |

The step automatically updates `steps.<name>` in the `contexts` engine with `conclusion`, `outcome`, and `outputs` after each run. Subsequent steps can reference `${{ steps.<name>.outputs.my_output }}`.

### Shell default

On Unix, the default shell is `bash -xe`. On Windows, `pwsh`. Supported values: `bash`, `sh`, `pwsh`, `powershell`, `python`, `cmd`.

---

## Sequencing steps

Steps must be chained via their `continue` output (for steps that allow failure) or `completed` output (for steps that must succeed). The typical pattern:

```mel
step1: runAction[contexts=contexts](name="step1", commands="echo hello")
step2: runAction[contexts=contexts](name="step2", commands="echo world")
step3: runAction[contexts=contexts](name="step3", commands="echo done", if="always()")

trigger -> ... -> step1.trigger
step1.completed -> step2.trigger
step2.completed -> step3.trigger

// collect failures
one<void>()
step1.failed -> one.a
step2.failed -> one.b
```

For steps with `continue_on_error`, chain `continue` instead of `completed` to let execution proceed:

```mel
step1: runAction[contexts=contexts](name="step1", commands="...", continue_on_error="true")
step2: runAction[contexts=contexts](name="step2", commands="...")

trigger -> step1.trigger
step1.continue -> step2.trigger
step1.failed -> Self.failed
```

---

## Job dispatch — `simpleStep`

For container-based jobs (equivalent to `runs-on: ubuntu-latest` with a docker image), use `simpleStep`. It manages the full lifecycle: spawning a runner, executing commands, reporting status, and stopping the runner.

```mel
treatment myJob[dispatcher: CicdDispatchEngine](
    github_token: string,
    github_project: string,
    github_sha: string
)
  input trigger: Block<void>
  output completed: Block<void>
  output failed: Block<void>
{
    simpleStep[dispatcher=dispatcher](
        name="my-job",
        image="rust:latest",
        cpu=1000,
        memory=2000,
        storage=10000,
        commands=[
            |command("cargo", ["test"]),
        ],
        report=true,
        github=true,
        github_token=github_token,
        github_project=github_project,
        github_sha=github_sha
    )

    Self.trigger -> simpleStep.trigger
    simpleStep.completed -> Self.completed
    simpleStep.failed -> Self.failed
}
```

### `simpleStep` key parameters

| Parameter | Default | Purpose |
|---|---|---|
| `name` | required | Step identifier |
| `image` | required | Container image |
| `cpu` | `500` | Millicores (1000 = one full CPU) |
| `memory` | `500` | MB |
| `storage` | `5000` | MB |
| `max_duration` | `3600` | Seconds before the worker is terminated |
| `arch` | `_` | Target architecture (`Option<Arch>`) |
| `service_containers` | `[]` | Side-car containers |
| `variables` | `_` | Environment variables (`Option<StringMap>`) |
| `commands` | required | List of `Command` to run sequentially |
| `out_file` | `_` | File to stream back through `data` output after success |
| `report` | `true` | Enable service state reporting |
| `github` | `false` | Report to GitHub commit status API |
| `github_token` | `""` | GitHub token |
| `github_project` | `""` | `owner/repo` |
| `github_sha` | `""` | Commit SHA |

### Local step — `localStep`

For steps that run directly on the Mélodium engine host (equivalent to `runs-on: self-hosted` without containers):

```mel
localStep(
    name="lint",
    commands=[|raw_commands("cargo clippy -- -D warnings")],
    report=true,
    github=true,
    github_token=github_token,
    github_project=github_project,
    github_sha=github_sha
)
```

`localStep` does not require a `CicdDispatchEngine` model.

---

## Status reporting

### With explicit credentials — `postGithubState`

Use when credentials are available as treatment parameters:

```mel
postGithubState(
    token=github_token,
    project=github_project,
    sha=github_sha,
    state=|pending(),
    name="my-check",
    description="Build started"
)
```

`state` is a `StepState` value, constructed with functions `|pending()`, `|success()`, `|failure()`, `|error()`.

The treatment automatically includes a `target_url` pointing to the Mélodium execution dashboard.

### With context expressions — `postGithubStateContext`

Use inside a job treatment that already has an initialised `contexts` engine:

```mel
postGithubStateContext[contexts=contexts](
    state=|pending(),
    name="my-check",
    description="Build started"
)
```

The API URL and `Authorization` header are resolved by evaluating `${{ github.api_url }}` and `${{ secrets.GITHUB_TOKEN }}` through the engine.

---

## Job result collection — `githubJobResult`

After all steps have run, collect the job outcome and compute outputs:

```mel
result: githubJobResult[contexts=contexts](
    name="my-job",
    outputs=|map([
        |entry("artifact_url", "${{ steps.upload.outputs.url }}")
    ]),
    local_context=|null()
)

// wire after all steps
lastStep.completed -> result.trigger_release

result.success -> Self.success
result.failure -> Self.failure
result.result -> Self.job_result
```

`githubJobResult` evaluates output expressions, queries `job.status` from the JS engine, and emits `success` or `failure` accordingly.

---

## Multi-job workflows

Jobs depending on other jobs use the `needs` mechanism. Chain jobs sequentially and inject the result into the downstream job's context using `includeNeeds`.

```mel
treatment pipeline[dispatcher: CicdDispatchEngine](github_contexts: string)
  input trigger: Block<void>
  output completed: Block<void>
  output failed: Block<void>
{
    contexts_a: JavaScriptEngine()
    contexts_b: JavaScriptEngine()

    prepA: prepareContexts[contexts=contexts_a](github_contexts=github_contexts)
    // run job A steps...
    resultA: githubJobResult[contexts=contexts_a](name="job-a", outputs=|map([]), local_context=|null())

    prepB: prepareContexts[contexts=contexts_b](github_contexts=github_contexts)
    injectNeeds: includeNeeds[contexts=contexts_b](from="job-a")
    // run job B steps...

    Self.trigger -> prepA.trigger,ready -> ...
    resultA.result -> injectNeeds.needs
    resultA.finished -> prepB.trigger,ready -> injectNeeds.trigger,ready -> ...
}
```

Each job gets its own `JavaScriptEngine` instance. The parent job's result `Json` is passed to `includeNeeds.needs`, making `${{ needs.job-a.outputs.* }}` and `${{ needs.job-a.result }}` available in job B.

---

## Workflow inputs — `includeInputs`

For `workflow_dispatch` or reusable workflows that receive inputs:

```mel
injectInputs: includeInputs[contexts=contexts]()

// inputs is a Block<Json> containing the parsed inputs object
Self.trigger -> injectInputs.trigger
parsed_inputs -> injectInputs.inputs
injectInputs.ready -> ...
```

After injection, `${{ inputs.my_input }}` is resolvable in all subsequent `runAction` calls.

---

## Action isolation — `replicateContextsWithInputs`

When implementing a reusable action (equivalent to a composite action), create a separate engine for the action scope, seeded from the parent context with resolved inputs:

```mel
action_contexts: JavaScriptEngine()

replicateContextsWithInputs[main_contexts=contexts, action_contexts=action_contexts](
    inputs=|map([
        |entry("token", github_token),
        |entry("ref", branch)
    ])
)

// then use action_contexts for runAction inside the action
```

This gives the action its own `inputs` variable while inheriting `github` and `runner` from the parent context. Input values in the `inputs` map are evaluated through `main_contexts` before injection (so they may contain `${{ }}` expressions).

---

## Complete job treatment pattern

This is the canonical pattern for migrating a single GitHub Actions job:

```mel
use javascript::JavaScriptEngine
use cicd/services/github::prepareContexts
use cicd/services/github::runAction
use cicd/services/github::githubJobResult
use cicd/runners::CicdDispatchEngine

treatment buildJob[dispatcher: CicdDispatchEngine](
    github_contexts: string,
    github_token: string,
    github_project: string,
    github_sha: string
)
  input trigger: Block<void>
  output success: Block<void>
  output failure: Block<void>
  output result: Block<Json>
  output finished: Block<void>
{
    contexts: JavaScriptEngine()
    prepare: prepareContexts[contexts=contexts](github_contexts=github_contexts)

    checkout: runAction[contexts=contexts](
        name="checkout",
        commands="git clone ${{ github.server_url }}/${{ github.repository }} . && git checkout ${{ github.sha }}"
    )

    build: runAction[contexts=contexts](
        name="build",
        commands="cargo build --release"
    )

    test: runAction[contexts=contexts](
        name="test",
        commands="cargo test"
    )

    result: githubJobResult[contexts=contexts](
        name="build-job",
        outputs=|map([]),
        local_context=|null()
    )

    oneFailed: one<void>()

    Self.trigger -> prepare.trigger,ready -> checkout.trigger,completed -> build.trigger,completed -> test.trigger
    test.completed -> result.trigger_release
    test.failed -> oneFailed.a
    build.failed -> oneFailed.b
    checkout.failed -> oneFailed.c

    result.success -> Self.success
    result.failure -> Self.failure
    result.result -> Self.result
    result.finished -> Self.finished
}
```

---

## Runner infrastructure

### `CicdDispatchEngine`

The dispatcher spawns remote workers.

```mel
model myDispatcher: CicdDispatchEngine(
    location="api",           // "api" (default) or "compose" for local testing
    api_token=_,              // uses engine default if none
    api_url=_                 // uses engine default if none
)
```

### `CicdRunnerEngine`

Used internally by `simpleStep` and `stepOn*` — not normally instantiated directly. It wraps `DistributionEngine` preconfigured to run `cicd/steps::step`.

### Direct step control with `stepOn`

For custom runner configurations, use `stepOn` directly instead of `simpleStep`:

```mel
runner: CicdRunnerEngine()

setupRunner[dispatcher=dispatcher, runner=runner](
    name="my-runner",
    cpu=1000,
    memory=2000,
    storage=10000,
    containers=[|container("main", 1000, 2000, 10000, |arm64(), [|mount("data", "/mnt/data")], "rust:latest", _)]
)

stepOn[runner=runner](
    name="build",
    executor_name=|wrap<string>("main"),
    commands=[|command("cargo", ["build"])],
    environment=|wrap<Environment>(|environment(|map([]), _, true, false))
)

stopRunner[runner=runner]()

Self.trigger -> setupRunner.trigger,ready -> stepOn.trigger,finished -> stopRunner.trigger
```

---

## Environment and commands

Commands are `Vec<Command>` built with functions from `process/command`:

```mel
use process/command::|command
use process/command::|raw_commands

commands=[
    |command("cargo", ["build", "--release"]),
    |raw_commands("echo done && ls -la"),
]
```

Environment is `Option<Environment>` built with:

```mel
use process/environment::Environment
use process/environment::|environment
use std/data/string_map::|map
use std/data/string_map::|entry

environment=|wrap<Environment>(|environment(
    |map([|entry("RUST_LOG", "info")]),  // variables
    _,                                    // working directory (Option<string>)
    true,                                 // inherit parent env
    false                                 // pass as /usr/bin/env wrapper
))
```

⚠️ Alpine-based images require `apk add coreutils-env` if `working_directory` is used, since the default `env` command does not support setting working directories.

---

## Matrix jobs

GitHub Actions `strategy.matrix` becomes parallel treatment instances wired into a `one<void>()` fan-in for completion collection. Each matrix cell is a separate treatment invocation:

```mel
// matrix: { os: [linux, mac], rust: [stable, nightly] }
// becomes four parallel steps, all driving the same completion gate

cellLinuxStable: simpleStep[dispatcher=dispatcher](name="test-linux-stable", image="rust:latest", ...)
cellLinuxNightly: simpleStep[dispatcher=dispatcher](name="test-linux-nightly", image="rustlang/rust:nightly", ...)
cellMacStable: localStep(name="test-mac-stable", ...)
cellMacNightly: localStep(name="test-mac-nightly", ...)

oneComplete: one<void>()
oneFailed: one<void>()

Self.trigger -> cellLinuxStable.trigger
Self.trigger -> cellLinuxNightly.trigger
Self.trigger -> cellMacStable.trigger
Self.trigger -> cellMacNightly.trigger

cellLinuxStable.success ---> oneComplete.a
cellLinuxNightly.success --> oneComplete.b
cellMacStable.success -----> oneComplete.c
cellMacNightly.success ----> oneComplete.d

cellLinuxStable.failed ----> oneFailed.a
cellLinuxNightly.failed ---> oneFailed.b
cellMacStable.failed ------> oneFailed.c
cellMacNightly.failed -----> oneFailed.d

oneComplete.value -> Self.success
oneFailed.value ---> Self.failed
```

ℹ️ Unlike YAML matrix, Mélodium parallelism is explicit and typed. Each cell can have a different image, resource allocation, or command set — no string interpolation required.

---

## Third-party actions — `uses:`

GitHub Actions `uses:` steps (marketplace actions) have no automatic equivalent. They must be reimplemented as Mélodium treatments. The `replicateContextsWithInputs` treatment provides the isolation pattern for action-scoped engines when inputs need expression evaluation:

```mel
// uses: actions/checkout@v4
// with:
//   ref: ${{ github.sha }}
//   token: ${{ secrets.GITHUB_TOKEN }}

checkoutAction: runAction[contexts=contexts](
    name="checkout",
    commands="""
        git init
        git remote add origin ${{ github.server_url }}/${{ github.repository }}
        git fetch --depth=1 origin ${{ github.sha }}
        git checkout FETCH_HEAD
    """,
    shell="bash"
)
```

For composite actions that bundle multiple steps, create a dedicated treatment with its own `JavaScriptEngine` and call `replicateContextsWithInputs` to seed it from the parent context.

---

## Step output semantics

All step treatments use a consistent set of outputs:

| Output | Meaning |
|---|---|
| `started` | First command began executing |
| `completed` | All commands finished (any exit code) |
| `success` | Commands exited with code `0` |
| `error` | Commands exited with non-zero code |
| `failed` | Executor failed to run commands at all |
| `finished` | Execution ended in any case |
| `terminated` | Cancelled via `terminate` input (terminable variants only) |

For `runAction` specifically: `completed` means exit code `0`, `failed` means non-zero exit or executor failure (unless `continue_on_error` is set).

---

## What Mélodium makes easier than YAML

**True parallelism without matrix syntax.** Any number of treatments can share the same `trigger` input. No `strategy.matrix` bookkeeping, no interpolated names, no `fail-fast` flag — just wire outputs to the same `one<void>()` fan-in.

**Streaming artifacts without upload/download.** The `data: Stream<byte>` output of one step can be piped directly into `simpleStepWithInput.data` of the next. No artifact storage, no size limits imposed by the CI platform, no separate upload/download steps.

**Type-safe parameters.** Secrets and configuration values are typed treatment parameters (`string`, `u32`, `bool`, `Option<T>`), not stringly-typed YAML variables that silently coerce. A missing required parameter is a compile error, not a runtime blank string.

**Conditional logic without expression hacks.** GitHub Actions `if:` expressions are string-evaluated JavaScript. In Mélodium, conditions are `filterBlock<void>()` and `passBlock<void>()` wired at the data-flow level — no quoting rules, no `fromJSON()` workarounds, no `${{ }}` injection risk.

**Reusable jobs as first-class treatments.** A treatment can be called from anywhere in the codebase with the same syntax as a built-in. No `workflow_call` boilerplate, no separate reusable workflow file, no input/output schema declaration separate from the implementation.

**Dynamic job counts.** YAML matrix size is fixed at parse time. Mélodium treatments can produce dynamic fan-outs by streaming triggers from a list, enabling job counts determined at runtime.

**Step outputs without file hacks.** GitHub Actions step outputs require writing to `$GITHUB_OUTPUT` (a file path injected as an environment variable). In Mélodium, `githubSetOutputs` reads that file automatically and injects the values into the `contexts` engine — and within pure Mélodium treatments, data flows directly through typed outputs without any file intermediary.
