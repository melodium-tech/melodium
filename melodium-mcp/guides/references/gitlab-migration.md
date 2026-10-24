# GitLab CI Migration Reference

This reference covers the patterns, treatments, and data types needed to migrate GitLab CI pipelines to Mélodium using the `cicd` package.

---

## Concept mapping

| GitLab CI | Mélodium |
|---|---|
| Pipeline (`.gitlab-ci.yml`) | `.mel` treatment |
| Job | Treatment wired with `simpleStep` or `localStep` |
| `script:` | `Vec<Command>` passed to step treatments |
| `before_script:` / `after_script:` | Commands prepended/appended to the `commands` list |
| Stage ordering | Sequential wiring between treatments |
| `needs:` / `dependencies:` | Sequential wiring; artifact data flows via `data` stream |
| `variables:` | Parameters passed to treatments |
| Masked or protected variables, `CI_JOB_TOKEN` | `Secret<string>` parameters, see [Secrets](#secrets) |
| `image:` | `image` parameter of `simpleStep` |
| `services:` | `service_containers` parameter of `simpleStep` |
| `artifacts:` | `out_file` + `data` stream output; `simpleStepWithInput` to consume |
| `cache:` | `cache` parameter of `simpleStep`, `\|wrap<Cache>(\|cache("data", "<dir under /mnt/data>", key, restore_keys))` from `work/cache`, restored before `commands` and saved after success, on Cadence.CI workers |
| `rules:` / `only:` / `except:` | `filterBlock<void>()` or `passBlock<void>()` on the `trigger` |
| `when: on_success` | Default: chain `success` output to next trigger |
| `when: on_failure` | Chain `failed` or `error` output to next trigger |
| `when: always` | `one<void>()` fan-in of both success and failure outputs |
| `when: never` | Simply omit the wiring |
| `when: manual` | Treatment input driven by an external event / human approval treatment |
| `allow_failure: true` | Handle `error` output instead of stopping the chain |
| `retry:` | Wrap step in a retry loop treatment |
| `timeout:` | `max_duration` parameter of `simpleStep` / `setupRunner` |
| `interruptible: true` | Use `stepOnTerminable` / `simpleStepTerminable` variants |
| `parallel:` | Multiple parallel treatment instances wired with `one<void>()` fan-in |
| `tags:` | `tags` parameter of `setupRunner` |
| `resource_group:` | No direct equivalent in current `CicdDispatchEngine` |
| `environment:` | Deployment protection rules are outside Mélodium scope |
| `trigger:` | Nest the downstream pipeline treatment call directly |
| `extends:` / `include:` | Mélodium `use` imports and treatment composition |
| `coverage:` | Parse coverage from command output within the `commands` list |
| Status reporting | `postGitlabState` / `setServiceState` |

---

## Package imports

```mel
use cicd::StepState
use cicd/services::setServiceState
use cicd/services/gitlab::postGitlabState
use cicd/naive::simpleStep
use cicd/naive::simpleStepWithInput
use cicd/naive::localStep
use cicd/runners::CicdDispatchEngine
use cicd/runners::CicdRunnerEngine
use cicd/runners::setupRunner
use cicd/runners::stopRunner
use cicd/steps::stepOn
use cicd/steps::stepOnWithInput
use cicd/steps::stepOnTerminable
use process/command::|command
use process/command::|raw_commands
use process/environment::Environment
use process/environment::|environment
use std/data/string_map::|map
use std/data/string_map::|entry
use work/resources::|container
use work/resources::|volume
use work/resources::|mount
```

---

## Secrets

CI/CD variables holding credentials (masked or protected variables, `CI_JOB_TOKEN`) become `Secret<string>` parameters. A secret holds where its value comes from, such as `env:MELODIUM_SECRET_GITLAB_TOKEN`, and only the elements using it reveal it: its value never appears in parameters, logs, reports or command arguments.

`.gitlab-ci.yml` passes locators, never values. GitLab gives CI/CD variables to the job as environment variables, and programs only get those whose name starts with `MELODIUM_SECRET_`, so the job gives each one under that prefix:

```yaml
variables:
  MELODIUM_SECRET_GITLAB_TOKEN: $GITLAB_TOKEN
script:
  - melodium run .melodium/Compo.toml main --gitlab_token env:MELODIUM_SECRET_GITLAB_TOKEN --project "$CI_PROJECT_ID"
```

Secret parameters given to models (and through them, as the status tokens of steps) are `const`. The status token of steps and `setServiceState`, `gitlab_token`, defaults to `env:MELODIUM_SECRET_GITLAB_TOKEN`.

Commands get secrets as environment variables with `secret_variables`, never through their arguments. Steps running on workers need secrets allowed to be sent to them by value, which `std/secret::|locate` sets explicitly. Secrets with the default policy stay on the engine holding them, and sending them to a worker fails with an explicit error.

```mel
use std/data/map::Map
use std/data/map::|map as |secret_map
use std/data/map::|entry as |secret_entry
use std/secret::|locate

simpleStep[dispatcher=dispatcher](
    name="publish",
    image="node:20",
    commands=[|command("npm", ["publish"])],
    secret_variables=|wrap<Map>(|secret_map([
        |secret_entry<Option<Secret<string>>>("NPM_TOKEN", |locate<string>("env:MELODIUM_SECRET_NPM_TOKEN", "npm_token", "value", false))
    ]))
)
```

### `CI_JOB_TOKEN` in repository URLs

`CI_REPOSITORY_URL` holds the job token, which `git clone` would then show in its arguments. Clone from `$CI_PROJECT_URL.git`, without credentials, and give git the token through secret variables (`GIT_CONFIG_*` needs git 2.31 or later), the job giving it as `MELODIUM_SECRET_CI_JOB_TOKEN: $CI_JOB_TOKEN` in its `variables`:

```mel
use std/secret::|format as |secret_format
use std/secret::|base64

simpleStep[dispatcher=dispatcher](
    name="build",
    image="rust:latest",
    variables=|wrap<StringMap>(|map([
        |entry("GIT_CONFIG_COUNT", "1"),
        |entry("GIT_CONFIG_KEY_0", "http.extraHeader")
    ])),
    secret_variables=|wrap<Map>(|secret_map([
        |secret_entry<Secret<string>>("GIT_CONFIG_VALUE_0", |secret_format(
            "Authorization: Basic {credentials}",
            |secret_map([|secret_entry<Secret<string>>("credentials", |base64(
                |secret_format(
                    "gitlab-ci-token:{token}",
                    |secret_map([|secret_entry<Option<Secret<string>>>("token", |locate<string>("env:MELODIUM_SECRET_CI_JOB_TOKEN", "job_token", "value", false))]),
                    "job_credentials"
                ),
                "job_credentials_base64"
            ))]),
            "git_authorization"
        ))
    ])),
    commands=[|command("git", ["clone", "--depth", "1", repository_url, "project"])]
)
```

---

## Running a job — `simpleStep`

`simpleStep` is the primary treatment for container-based jobs (`image:` in GitLab CI). It manages the full lifecycle: spawning a runner, executing commands, reporting status to GitLab, and stopping the runner.

```mel
treatment buildJob[dispatcher: CicdDispatchEngine](
    const gitlab_token: Secret<string>,
    gitlab_project_id: string,
    gitlab_sha: string,
    gitlab_ref: string,
    gitlab_pipeline_id: string
)
  input trigger: Block<void>
  output completed: Block<void>
  output failed: Block<void>
{
    simpleStep[dispatcher=dispatcher](
        name="build",
        image="rust:latest",
        cpu=1000,
        memory=2000,
        storage=10000,
        commands=[
            |command("cargo", ["build", "--release"])
        ],
        report=true,
        gitlab=true,
        gitlab_token=gitlab_token,
        gitlab_project_id=gitlab_project_id,
        gitlab_sha=gitlab_sha,
        gitlab_ref=gitlab_ref,
        gitlab_pipeline_id=gitlab_pipeline_id,
        description="Building the project"
    )

    Self.trigger -> simpleStep.trigger
    simpleStep.completed -> Self.completed
    simpleStep.failed -> Self.failed
}
```

### `simpleStep` key parameters

| Parameter | Default | Purpose |
|---|---|---|
| `name` | required | Job identifier shown in GitLab UI |
| `image` | required | Container image (equivalent to `image:`) |
| `cpu` | `500` | Millicores (1000 = one full CPU) |
| `memory` | `500` | MB of RAM |
| `storage` | `5000` | MB of disk |
| `max_duration` | `3600` | Seconds before the worker is terminated |
| `arch` | `_` | Target architecture (`Option<Arch>`) |
| `service_containers` | `[]` | Side-car containers (equivalent to `services:`) |
| `variables` | `_` | Environment variables (`Option<StringMap>`) |
| `secret_variables` | `_` | Environment variables whose values are secrets (`Option<Map>` of `Secret<string>`), see [Secrets](#secrets) |
| `pull_secret` | `_` | Registry credentials as a Docker configuration JSON (`Option<Secret<string>>`) |
| `commands` | required | List of `Command` to run sequentially |
| `out_file` | `_` | File to stream back as `data` after success (equivalent to `artifacts:`) |
| `out_storage` | `_` | Override data volume size in MB |
| `cache` | `_` | Directory of the `data` volume restored from a Cadence.CI cache before `commands` and saved under its key after success (`Option<Cache>`, from `work/cache::\|cache`), equivalent to `cache:` |
| `report` | `true` | Enable service state reporting |
| `gitlab` | `false` | Report to GitLab commit status API |
| `gitlab_root_url` | `"https://gitlab.com/api/v4"` | GitLab API root URL |
| `gitlab_token` | `"env:MELODIUM_SECRET_GITLAB_TOKEN"` | GitLab token allowed to set commit statuses (`const Secret<string>`) |
| `gitlab_project_id` | `""` | GitLab project ID |
| `gitlab_sha` | `""` | Commit SHA |
| `gitlab_ref` | `""` | Branch or tag ref |
| `gitlab_pipeline_id` | `""` | Pipeline ID |
| `description` | `""` | Human-readable status description |
| `log_service_response` | `false` | Log the raw API response |

### `simpleStep` outputs

| Output | Meaning |
|---|---|
| `started` | First command began |
| `completed` | All commands finished |
| `success` | Commands exited with code `0` |
| `error` | Commands exited with non-zero code |
| `failed` | Executor failed to run commands at all |
| `finished` | Execution ended in any case |
| `data` | Byte stream of `out_file` if set |

---

## Local jobs — `localStep`

For jobs that run directly on the Mélodium engine host (equivalent to `tags: [self-hosted]` without a container):

```mel
localStep(
    name="lint",
    commands=[|raw_commands("cargo clippy -- -D warnings")],
    report=true,
    gitlab=true,
    gitlab_token=gitlab_token,
    gitlab_project_id=gitlab_project_id,
    gitlab_sha=gitlab_sha,
    gitlab_ref=gitlab_ref,
    gitlab_pipeline_id=gitlab_pipeline_id
)
```

`localStep` does not require a `CicdDispatchEngine` model. It runs commands directly on the host with an optional `variables` `StringMap` and `secret_variables` as environment.

---

## Status reporting — `postGitlabState`

For direct, one-shot status posts (outside of `simpleStep`/`localStep`):

```mel
postGitlabState(
    root_url="https://gitlab.com/api/v4",
    token=gitlab_token,
    project=gitlab_project_id,
    sha=gitlab_sha,
    state=|pending(),
    ref=gitlab_ref,
    pipeline=gitlab_pipeline_id,
    name="my-check",
    description="Pipeline started"
)
```

`token` is a `const Secret<string>`, sent as the `PRIVATE-TOKEN` header when the request is made.

`state` is a `StepState` value built with functions from `cicd/services/gitlab`:

| Function | GitLab state string |
|---|---|
| `|pending()` | `"pending"` |
| `|running()` | `"running"` |
| `|success()` | `"success"` |
| `|failed()` | `"failed"` |
| `|canceled()` | `"canceled"` |
| `|skipped()` | `"skipped"` |

The treatment always includes a `target_url` pointing to the Mélodium execution dashboard.

---

## Lifecycle reporting — `setServiceState`

`setServiceState` ties the step lifecycle signals (`pending`, `running`, `success`, `error`, `failed`, `canceled`) directly to service API calls. It is used internally by `simpleStep` and `localStep`, but can also be used manually for custom runner setups:

```mel
setServiceState(
    name="build",
    report=true,
    gitlab=true,
    gitlab_token=gitlab_token,
    gitlab_project_id=gitlab_project_id,
    gitlab_sha=gitlab_sha,
    gitlab_ref=gitlab_ref,
    gitlab_pipeline_id=gitlab_pipeline_id
)

Self.trigger --------------> setServiceState.pending
spawnCommands.started ----> setServiceState.running
spawnCommands.success ----> setServiceState.success
spawnCommands.error ------> setServiceState.error
spawnCommands.failed -----> setServiceState.failed
spawnCommands.terminated -> setServiceState.canceled
```

Note: GitLab maps both `error` and `failed` to the `failed` state. `canceled` maps to `canceled`.

---

## Stage ordering

GitLab CI stages become sequential wiring between treatments. Each job treatment exposes `completed`/`failed` outputs that drive the next job's `trigger`:

```mel
treatment pipeline[dispatcher: CicdDispatchEngine](
    const gitlab_token: Secret<string>,
    gitlab_project_id: string,
    gitlab_sha: string,
    gitlab_ref: string,
    gitlab_pipeline_id: string
)
  input trigger: Block<void>
  output success: Block<void>
  output failed: Block<void>
{
    build: simpleStep[dispatcher=dispatcher](
        name="build",
        image="rust:latest",
        cpu=1000, memory=2000, storage=10000,
        commands=[|command("cargo", ["build", "--release"])],
        report=true, gitlab=true,
        gitlab_token=gitlab_token,
        gitlab_project_id=gitlab_project_id,
        gitlab_sha=gitlab_sha, gitlab_ref=gitlab_ref,
        gitlab_pipeline_id=gitlab_pipeline_id
    )

    test: simpleStep[dispatcher=dispatcher](
        name="test",
        image="rust:latest",
        cpu=1000, memory=2000, storage=10000,
        commands=[|command("cargo", ["test"])],
        report=true, gitlab=true,
        gitlab_token=gitlab_token,
        gitlab_project_id=gitlab_project_id,
        gitlab_sha=gitlab_sha, gitlab_ref=gitlab_ref,
        gitlab_pipeline_id=gitlab_pipeline_id
    )

    oneFailed: one<void>()
    build.failed -> oneFailed.a
    test.failed ---> oneFailed.b

    Self.trigger -> build.trigger,success -> test.trigger
    test.success -> Self.success
    oneFailed.value -> Self.failed
}
```

---

## Passing artifacts between jobs

GitLab CI artifacts passed between jobs map to the `data` stream output. Use `simpleStepWithInput` to receive the upstream artifact and write it into the container before running commands.

```mel
// Job A produces an artifact
buildStep: simpleStep[dispatcher=dispatcher](
    name="build",
    image="rust:latest",
    commands=[|command("cargo", ["build", "--release"])],
    out_file="target/release/myapp"
    // ...other params
)

// Job B consumes it
deployStep: simpleStepWithInput[dispatcher=dispatcher](
    name="deploy",
    image="alpine:latest",
    commands=[|raw_commands("./deploy.sh")],
    in_file="myapp",
    // ...other params
)

buildStep.data -> deployStep.data
buildStep.success -> deployStep.trigger
```

`in_file` names the file as it will appear inside the container at `/mnt/data/<in_file>`. `out_file` names the file the container writes to `/mnt/data/<out_file>` that will be streamed back.

---

## `allow_failure` jobs

For a job equivalent to `allow_failure: true`, connect the downstream trigger to both `success` and `error` (or `failed`) outputs:

```mel
optionalStep: simpleStep[dispatcher=dispatcher](
    name="optional-lint",
    image="rust:latest",
    commands=[|raw_commands("cargo clippy")],
    // ...
)

oneNext: one<void>()
optionalStep.success -> oneNext.a
optionalStep.error ---> oneNext.b  // continue even on non-zero exit

oneNext.value -> nextJob.trigger
```

---

## `when: always` jobs

For jobs that must run regardless of previous job outcome (equivalent to `when: always`), connect both success and failure outputs of the preceding job to the trigger:

```mel
alwaysStep: simpleStep[dispatcher=dispatcher](
    name="cleanup",
    // ...
)

oneAlways: one<void>()
previousStep.success -> oneAlways.a
previousStep.failed ---> oneAlways.b
previousStep.error ----> oneAlways.c

oneAlways.value -> alwaysStep.trigger
```

---

## Custom runner configuration

For jobs that need specific container arrangements beyond what `simpleStep` provides, use `setupRunner` + `stepOn` + `stopRunner` directly:

```mel
runner: CicdRunnerEngine()

setupRunner[dispatcher=dispatcher, runner=runner](
    name="custom-job",
    cpu=2000,
    memory=4000,
    storage=20000,
    volumes=[|volume("workspace", 5000)],
    containers=[
        |container(
            "main",
            4000, 2000, 10000,  // memory, cpu, storage
            |wrap<Arch>(|arm64()),
            [|mount("workspace", "/workspace")],
            "rust:latest",
            _
        )
    ],
    service_containers=[
        |serviceContainer("postgres", "postgres:15", |map([|entry("POSTGRES_PASSWORD", "test")]))
    ]
)

stepOn[runner=runner](
    name="custom-job",
    executor_name=|wrap<string>("main"),
    commands=[
        |command("cargo", ["test", "--features", "integration"])
    ],
    environment=|wrap<Environment>(|environment(
        |map([|entry("DATABASE_URL", "postgres://postgres:test@localhost/test")]),
        _,
        true,
        false
    )),
    out_file=|wrap<string>("results.tar")
)

stopRunner[runner=runner]()

Self.trigger -> setupRunner.trigger,ready -> stepOn.trigger,finished -> stopRunner.trigger
stepOn.success -> Self.success
setupRunner.failed -> Self.failed
stepOn.failed -------> Self.failed
```

---

## Environment and commands

Commands are `Vec<Command>` built with functions from `process/command`:

```mel
use process/command::|command
use process/command::|raw_commands

// Structured command (binary + arguments)
|command("cargo", ["build", "--release"])

// Shell string (runs through sh -c)
|raw_commands("echo hello && ls -la /workspace")
```

Environment variables are `Option<Environment>` built with:

```mel
use process/environment::|environment

|wrap<Environment>(|environment(
    |map([
        |entry("CI", "true"),
        |entry("RUST_BACKTRACE", "1")
    ]),
    _,      // working directory: Option<string>
    true,   // inherit parent environment
    false   // wrap with /usr/bin/env (needed for some containers)
))
```

⚠️ Alpine-based images require `apk add coreutils-env` if `working_directory` is used, since the default `env` command does not support setting working directories.

---

## Runner infrastructure

### `CicdDispatchEngine`

The dispatcher spawns remote worker processes.

```mel
model myDispatcher: CicdDispatchEngine(
    location="api",   // "api" (production) or "compose" (local testing with podman/docker compose)
    api_token=_,      // Option<Secret<string>>: the MELODIUM_API_TOKEN environment variable if none
    api_url=_         // uses the engine's own API URL if none
)
```

### Resource helpers

```mel
use work/resources::|container
use work/resources::|serviceContainer
use work/resources::|volume
use work/resources::|mount
use work/resources/arch::|arm64

// Container definition
|container(
    name,           // string: executor name
    memory,         // u32: MB
    cpu,            // u32: millicores
    storage,        // u32: MB
    |wrap<Arch>(|arm64()),  // Option<Arch>: none (_) means the architecture of the worker
    [|mount("vol-name", "/path/in/container")],
    "image:tag",    // string: container image
    _               // Option<Secret<string>>: pull secret, registry credentials as a Docker configuration JSON
)

// Service container (equivalent to GitLab services:)
|serviceContainer(
    "service-name",
    "postgres:15",
    |map([|entry("POSTGRES_PASSWORD", "secret")])
)

// Volume shared between engine and containers
|volume("vol-name", 1000)  // name, size in MB

// Mount a volume at a path inside a container
|mount("vol-name", "/mnt/data")
```

### Worker needs: architecture and system

`arch` and `system` are both optional: leave them out (`_`) to get any worker with the requested resources. Every need given is a requirement, the worker is spawned satisfying it or the run is refused with a message naming the need.

`arch` (`Option<Arch>`, available on `simpleStep` and `setupRunner`) is built with the functions of `work/resources/arch`, with the OCI image specification names or the Rust ones:

| Function | Architecture | Information |
|---|---|---|
| `\|amd64()`, `\|x86_64()` | `amd64`, `x86_64` | 64-bit x86 (Intel, AMD). Most servers, cloud instances, PCs and Intel Macs. |
| `\|arm64()`, `\|aarch64()` | `arm64`, `aarch64` | 64-bit ARM. Apple Silicon Macs, AWS Graviton and other ARM cloud instances, Raspberry Pi with a 64-bit system. |
| `\|i386()`, `\|x86()`, `\|i686()` | `386`, `x86`, `i686` | 32-bit x86. Old PCs and legacy systems. |
| `\|arm()`, `\|armv7()` | `arm`, `armv7` | 32-bit ARM, hard float. Raspberry Pi with a 32-bit system, embedded boards. |
| `\|riscv64()`, `\|riscv64gc()` | `riscv64`, `riscv64gc` | 64-bit RISC-V. Development boards and emerging servers. |
| `\|ppc64le()`, `\|powerpc64le()` | `ppc64le`, `powerpc64le` | 64-bit PowerPC, little endian. IBM Power servers running Linux. |
| `\|ppc64()`, `\|powerpc64()` | `ppc64`, `powerpc64` | 64-bit PowerPC, big endian. Older IBM Power and legacy PowerPC systems. |
| `\|s390x()` | `s390x` | IBM Z mainframes, including LinuxONE. |
| `\|loong64()`, `\|loongarch64()` | `loong64`, `loongarch64` | 64-bit LoongArch. Recent Loongson processors. |
| `\|mips()` | `mips` | 32-bit MIPS, big endian. Legacy networking equipment. |
| `\|mipsle()`, `\|mipsel()` | `mipsle`, `mipsel` | 32-bit MIPS, little endian. Routers and embedded devices. |
| `\|mips64()` | `mips64` | 64-bit MIPS, big endian. Legacy networking equipment and servers. |
| `\|mips64le()`, `\|mips64el()` | `mips64le`, `mips64el` | 64-bit MIPS, little endian. Older Loongson processors and some network devices. |
| `\|wasm32()` | `wasm32`, `wasm` | 32-bit WebAssembly. Browsers, WASI runtimes and edge platforms. |
| `\|wasm64()` | `wasm64` | 64-bit WebAssembly. Experimental, few runtimes support it. |

Both names of an architecture give the same value: the first is the one of the OCI image specification, the second the one Rust uses for its targets.

All containers of one worker must use the same architecture. A container given no architecture (`_`) runs on the architecture of its worker. `|local_arch()` returns the architecture of the machine running the program, to request a worker of the same architecture. The local `compose` location only runs the architecture of the host and refuses any other.

`system` (`Option<System>`, available on `setupRunner`, not on `simpleStep`) is what the worker runs on, either an operating system or one exact machine image, never both. It is built with the functions of `work/resources/system`:

```mel
use std/ops/option::|wrap
use work/resources/system::System
use work/resources/system::|linux_distribution_version
use work/resources/system::|image

// Any Ubuntu 24.04 machine
setupRunner[dispatcher=dispatcher, runner=runner](
    name="build", cpu=2000, memory=4000, storage=20000,
    system=|wrap<System>(|linux_distribution_version("ubuntu", "24.04"))
)

// This exact machine image, whatever OS and architecture it holds
setupRunner[dispatcher=dispatcher, runner=runner](
    name="build", cpu=2000, memory=4000, storage=20000,
    system=|wrap<System>(|image("nixos-ci"))
)
```

| Function | System |
|---|---|
| `\|linux()`, `\|linux_distribution(name)`, `\|linux_distribution_version(name, version)` | Linux; the distribution is the `ID` of `/etc/os-release` (`"ubuntu"`, `"debian"`, `"nixos"`, ...) and the version its `VERSION_ID` (`"24.04"`) |
| `\|windows()`, `\|windows_version(version)` | Windows (`"2022"`) |
| `\|macos()`, `\|macos_version(version)` | macOS (`"15"`) |
| `\|freebsd()`, `\|openbsd()`, `\|netbsd()` | BSD systems |
| `\|solaris()`, `\|illumos()`, `\|aix()`, `\|ibmi()`, `\|zos()` | Other Unix and mainframe systems |

Each of `\|windows`, `\|macos`, `\|freebsd`, `\|openbsd`, `\|netbsd`, `\|solaris`, `\|illumos`, `\|aix`, `\|ibmi` and `\|zos` has a `_version(version)` variant.
| `\|image(name)` | One machine image, by name (never a container image); with `arch` given, the image must exist for it |

Not every system can be spawned by every provider: a request nobody can satisfy is refused. The local `compose` location only satisfies a bare `\|linux()`.

---

## `before_script` and `after_script`

GitLab CI `before_script` and `after_script` are prepended/appended to the `commands` list. There is no separate concept in Mélodium:

```mel
// before_script: [apt-get update, apt-get install -y curl]
// script: [cargo test]
// after_script: [echo done]

commands=[
    |raw_commands("apt-get update && apt-get install -y curl"),
    |command("cargo", ["test"]),
    |raw_commands("echo done")
]
```

ℹ️ Unlike GitLab CI, `after_script` does not run in a separate shell context — all commands share the same environment. Use `stepOnTerminable` if cleanup on cancellation is required.

---

## Conditional rules — `rules:` / `only:` / `except:`

Conditional job execution maps to `filterBlock<void>()` on the trigger, with the condition evaluated before the trigger reaches the step:

```mel
// rules:
//   - if: $CI_COMMIT_BRANCH == "main"

filterMain: filterBlock<void>()
emitBranch: emit<bool>(value=|equal<string>(branch, "main"))
stream<bool>()
trigger<bool>()

Self.trigger -> emitBranch.trigger,emit -> stream.block,stream -> trigger.stream,first -> filterMain.select
Self.trigger -> filterMain.value,accepted -> simpleStep.trigger
```

For complex rule sets (multiple conditions, `changes:`, `exists:`) evaluate the condition as a `bool` parameter passed in from the entry-point treatment, where the caller has already resolved the branch name, changed files, etc.

---

## Parallel jobs — `parallel:`

GitLab CI `parallel: N` becomes N explicit parallel treatment instances. Each gets a distinct `name` for status reporting:

```mel
// parallel: 3

shard1: simpleStep[dispatcher=dispatcher](name="test-1-3", commands=[|raw_commands("cargo test -- --test-thread=1 shard_1")], ...)
shard2: simpleStep[dispatcher=dispatcher](name="test-2-3", commands=[|raw_commands("cargo test -- --test-thread=1 shard_2")], ...)
shard3: simpleStep[dispatcher=dispatcher](name="test-3-3", commands=[|raw_commands("cargo test -- --test-thread=1 shard_3")], ...)

allDone: one<void>()
shard1.success -> allDone.a
shard2.success -> allDone.b
shard3.success -> allDone.c

Self.trigger -> shard1.trigger
Self.trigger -> shard2.trigger
Self.trigger -> shard3.trigger

allDone.value -> Self.success
```

---

## Interruptible jobs — `interruptible: true`

Use the `Terminable` variants of step treatments and wire an external cancellation signal to the `terminate` input:

```mel
interruptibleStep: simpleStepTerminable[dispatcher=dispatcher](
    name="long-build",
    // ...
)

Self.trigger -> interruptibleStep.trigger
Self.cancel -> interruptibleStep.terminate

interruptibleStep.success ---> Self.success
interruptibleStep.terminated -> Self.canceled
interruptibleStep.failed ---> Self.failed
```

---

## Downstream pipeline triggers — `trigger:`

GitLab CI `trigger:` launches a child pipeline. In Mélodium, just call the downstream pipeline treatment directly:

```mel
// trigger:
//   project: my-group/my-downstream
//   strategy: depend

downstreamPipeline[dispatcher=dispatcher](
    gitlab_token=gitlab_token,
    gitlab_project_id=downstream_project_id,
    gitlab_sha=downstream_sha,
    gitlab_ref=gitlab_ref,
    gitlab_pipeline_id=gitlab_pipeline_id
)

Self.trigger -> downstreamPipeline.trigger
downstreamPipeline.success -> Self.success
downstreamPipeline.failed -> Self.failed
```

Since treatments are composable, `strategy: depend` is the default behaviour — the parent waits for `success` or `failed` from the child treatment.

---

## What Mélodium makes easier than YAML

**Streaming artifacts without object storage.** GitLab CI artifacts are uploaded to object storage between jobs, incurring upload/download time and storage quotas. The `data: Stream<byte>` output of one step pipes directly into `simpleStepWithInput.data` of the next — zero intermediate storage, no size quota, no expiry policy.

**True parallelism without `parallel:` limits.** Any number of steps can share the same `trigger`. No need to declare a fixed `parallel: N` integer — fan out as many instances as needed, each with distinct names, images, and resource allocations.

**Type-safe variables.** GitLab CI `variables:` are always strings, leading to coercion bugs (`"true"` vs `true`, numeric overflow). Mélodium treatment parameters are typed (`bool`, `u32`, `Option<string>`) and checked at load time. Credentials are `Secret<string>`, revealed only by the elements using them and masked in logs, instead of masked strings anyone can echo.

**`before_script` / `after_script` without isolation surprises.** GitLab runs `after_script` in a fresh shell, which means `export`ed variables from `script:` are invisible. In Mélodium, all commands in a `commands` list share the same environment — there is no hidden shell boundary.

**Conditional wiring at the data-flow level.** `rules:` in GitLab CI is evaluated as YAML-interpreted expressions before the pipeline even starts. In Mélodium, `filterBlock<void>()` and `passBlock<void>()` are first-class flow treatments that can incorporate runtime values — branch names, file presence checks, API responses — not just static CI variables.

**Reusable jobs as treatments, not YAML anchors.** GitLab CI reuse relies on `extends:`, `include:`, and YAML anchors — all resolved at parse time. Mélodium `use` imports and treatment calls are typed and composable at the language level, with no YAML merge-key edge cases.

**Downstream pipelines as nested calls.** GitLab `trigger:` launches a child pipeline asynchronously and optionally waits with `strategy: depend`. In Mélodium, the downstream pipeline is just a treatment call — sequential or parallel, with full access to the parent's parameters and return values, no bridge API needed.

**No implicit stage ordering.** GitLab CI stages are a global ordering mechanism that applies to all jobs. Mélodium wiring is explicit per job — no surprise about which jobs are implicitly blocked by which stage.

---

## Complete pipeline treatment pattern

```mel
use cicd/runners::CicdDispatchEngine
use cicd/naive::simpleStep
use std/flow::one
use process/command::|command

treatment ciPipeline[dispatcher: CicdDispatchEngine](
    const gitlab_token: Secret<string>,
    gitlab_project_id: string,
    gitlab_sha: string,
    gitlab_ref: string,
    gitlab_pipeline_id: string
)
  input trigger: Block<void>
  output success: Block<void>
  output failed: Block<void>
{
    build: simpleStep[dispatcher=dispatcher](
        name="build",
        image="rust:latest",
        cpu=1000, memory=2000, storage=10000,
        commands=[|command("cargo", ["build", "--release"])],
        report=true, gitlab=true,
        gitlab_token=gitlab_token, gitlab_project_id=gitlab_project_id,
        gitlab_sha=gitlab_sha, gitlab_ref=gitlab_ref,
        gitlab_pipeline_id=gitlab_pipeline_id,
        description="Building"
    )

    test: simpleStep[dispatcher=dispatcher](
        name="test",
        image="rust:latest",
        cpu=2000, memory=4000, storage=10000,
        commands=[|command("cargo", ["test"])],
        report=true, gitlab=true,
        gitlab_token=gitlab_token, gitlab_project_id=gitlab_project_id,
        gitlab_sha=gitlab_sha, gitlab_ref=gitlab_ref,
        gitlab_pipeline_id=gitlab_pipeline_id,
        description="Testing"
    )

    deploy: simpleStep[dispatcher=dispatcher](
        name="deploy",
        image="alpine:latest",
        cpu=500, memory=500, storage=5000,
        commands=[|raw_commands("./deploy.sh")],
        report=true, gitlab=true,
        gitlab_token=gitlab_token, gitlab_project_id=gitlab_project_id,
        gitlab_sha=gitlab_sha, gitlab_ref=gitlab_ref,
        gitlab_pipeline_id=gitlab_pipeline_id,
        description="Deploying"
    )

    oneFailed: one<void>()
    build.failed -> oneFailed.a
    test.failed ---> oneFailed.b
    deploy.failed -> oneFailed.c

    Self.trigger -> build.trigger,success -> test.trigger,success -> deploy.trigger
    deploy.success -> Self.success
    oneFailed.value -> Self.failed
}
```
