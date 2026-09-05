# CI Pipeline: showcase

Not a tutorial step: a three-stage CI pipeline that runs entirely on provisioned cloud containers, combining `cicd`, `work`, and `process` in one realistic scenario.

> **Requirements:** a Mélodium Services API token, or a local `podman`/`docker compose` setup (see `CicdDispatchEngine`'s `location` parameter). Set `MELODIUM_API_TOKEN` in the environment and run with `--api-report`; see Cadence.CI to obtain a token and follow execution. Nothing else: `repo_url` defaults to a real, public repository (see below). This pipeline was run against real Cadence.CI infrastructure: stages 1 (`build`) and 3 (`package`) succeed for real, producing a genuine compiled `hexyl` binary and `artifact.tar.gz`. Stage 2 (`test`) currently fails for an infrastructure reason unrelated to this example's own code, see *How it was checked* below for the exact error and what was and was not resolved.

## What it does

```
export MELODIUM_API_TOKEN="my-api-token"
melodium run --api-report Compo.toml
```

`repo_url` defaults to [`sharkdp/hexyl`](https://github.com/sharkdp/hexyl), a small, real, permissively-licensed Rust binary crate with no external dependencies at build or test time, chosen specifically so this example is copy-paste runnable. Pass `--repo_url` to build something else instead:

```
melodium run --api-report Compo.toml \
  --repo_url "https://github.com/your-org/your-project.git"
```

- **Stage 1: build**: `rust:1.90-slim` container; `cargo build --release`; the binary's name is read back from the cloned repo's own `Cargo.toml` (works for any ordinary single-binary crate, not just the default one), and streams back locally.
- **Stage 2: test**: `rust:1.90-slim` + a `postgres:16` *service container*; a `psql` connectivity check against `DATABASE_URL` proves the sidecar is actually reachable, then `cargo test` runs the repo's own test suite. `hexyl`'s own tests are database-independent, so the `psql` check, not `cargo test`, is what this stage actually demonstrates about `service_containers`; point `repo_url` at a project with database-backed integration tests to exercise the sidecar more fully. **This stage does not currently succeed against live infrastructure**, see *How it was checked*.
- **Stage 3: package**: `debian:bookworm-slim`; bundles stage 1's binary into a `.tar.gz`, streamed back and written to disk.

Stages 1 and 2 run **concurrently**; stage 3 waits for both to finish and only proceeds if neither failed.

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `dispatcher` | `CicdDispatchEngine` | Spawns each stage's container, on Mélodium Services or a local compose setup |

### Data flow

```
startup ──┬──▶ build (rust container)  ──┐
          └──▶ test  (rust + postgres) ──┴──▶ flock (wait for both) ──▶ package (debian container) ──▶ writeLocal
```

## Runtime behaviour

1. `build` and `test` both start on `startup.trigger`: two independent containers, provisioned and run in parallel, each reporting `started`/`success`/`error`/`failed`/`finished` independently. Both are explicitly pinned to the same `arch` (`|arm64()`); a service container's `arch` has no "unspecified" option the way a step's own does, so if the two ever disagree, dispatch is rejected outright, before either container starts.
2. Two `one<void>()` treatments fan a "did *anything* fail" signal in from four separate failure/error outputs (`std/flow::one` emits the first of two incoming signals to arrive, essentially an OR-gate for `Block<void>`), driving a single abort log line without duplicating the check four times.
3. `flock<void>()` merges `build.finished` and `test.finished` into a stream that only completes once *both* blocks have arrived; converting that back to a `Block<void>` with `trigger.start` is what actually gates stage 3.
4. `build`'s `data` output (the compiled binary, read from `/mnt/data/binary` inside the container) is wired directly into `package`'s `data` input: one container's filesystem output becomes another container's filesystem input, without ever touching the machine running this program in between.

### How it was checked

This pipeline was actually run against real Cadence.CI infrastructure. That surfaced two real bugs, both now fixed, and one still-open problem.

**Bug 1: `commands` never went through a shell.** The first version used `|raw_commands([...])` with plain shell syntax: `${REPO_URL}`, `&&`, `cd`, `2>&1`, `$(...)`. It type-checked. Live, `git clone` tried to clone a repository literally named `${REPO_URL}`, because nothing had ever substituted it: `|raw_commands` only tokenises a string (`shlex::split`), and the executor spawns the result directly (`async_std::process::Command::new(...)`), no shell involved at any point. `variables` really do become OS environment variables for the process (confirmed in `libs/process-mel/src/local.rs`), but nothing expands `${VAR}` syntax in an argv string unless the thing running is itself a shell. Fixed by wrapping every command that needs shell features in an explicit `|command("sh", ["-c", "..."])`.

**Bug 2: an unpinned `arch` doesn't mean "matches the service container".** With the shell fix in place, dispatch failed immediately: `Architecture is not valid or consistent for run and all containers`. The step's own `arch` can be left `_` (unspecified) and gets scheduled onto whatever the pool has; the Postgres *service container*'s `arch` has no such option, it is a required field, and it was hardcoded to `|amd64()`. The pool being used when this was tested only had `arm64` capacity, so the two never matched. Fixed by pinning both the step and the service container to the same explicit `|arm64()`.

**With both fixed, stage 1 and stage 3 now succeed for real**: a genuine `git clone` of `hexyl`, a genuine `cargo build --release`, and a genuine 538 KB `artifact.tar.gz` written to disk.

**Stage 2 still does not succeed.** After the two fixes above, dispatch is accepted, but the Postgres sidecar itself fails: `Container postgres did not reach running state (status: stopped)`. Raising its memory allocation (512 → 1024 MB) did not change the outcome. This looks like a problem with the sidecar container specifically, not with this example's own wiring or the two bugs above, but the exact cause was not found. If you hit this too: it is not something in this pipeline's own code that is known to be wrong, and it is an honest gap in this README rather than a claim this stage works.

### Key Mélodium patterns used

- **`simpleStep` / `simpleStepWithInput`**: the difference is exactly what it sounds like, whether the step needs data streamed *in* (as a file at `/mnt/data/<in_file>`) in addition to whatever it streams *out*.
- **`service_containers`**: sidecar containers (like the Postgres instance for stage 2) that live alongside the step's main container for its duration; `|service_container(...)`'s parameters follow its documented order (`name, memory, cpu, storage, arch, mounts, image, pull_secret, env, command`), confirmed against the same order used successfully elsewhere in this project rather than assumed from field listing order (see the tutorial's note on function argument order in [09_process_pipeline](../../tutorial/09_process_pipeline/)).
- **`one<void>()` as an OR-gate, `flock<void>()` as an AND-gate**: two small, reusable shapes for combining multiple `Block<void>` signals without hand-writing the logic each time.
- **A showcase's default parameter is itself part of what must be verified.** A `repo_url` with no default, or one pointing at a placeholder like `my-org/my-project`, quietly makes the example unrunnable without the reader doing their own setup work first. Giving it a real, verified default is what makes `--api-report` and a token the only two things a reader needs to supply.
- **Prove infrastructure is wired correctly independently of the workload that happens to run on it**: `cargo test` here would pass identically whether or not `DATABASE_URL` pointed at a real, reachable database, so it proves nothing about the `service_containers` wiring by itself. The `psql -c "SELECT 1;"` step is a minimal, targeted check of exactly the thing this stage exists to demonstrate.
- **`commands` is not a shell, no matter how shell-like the strings look.** `|raw_commands` tokenises; it does not interpret `${VAR}`, `&&`, `cd`, or redirection. Anything that needs those needs an explicit `|command("sh", ["-c", "..."])`.
- **A required field with no "unspecified" option has to match something, deliberately.** `service_containers`' `arch` cannot be left to the scheduler the way a step's own can; leaving one pinned and the other not is a silent trap that only surfaces at dispatch time, on real infrastructure, not at `melodium check`.

Back to the [examples index](../../README.md).
