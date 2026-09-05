# CI Pipeline: showcase

Not a tutorial step: a three-stage CI pipeline that runs entirely on provisioned cloud containers, combining `cicd`, `work`, and `process` in one realistic scenario.

> **Requirements:** a Mélodium Services API token, or a local `podman`/`docker compose` setup (see `CicdDispatchEngine`'s `location` parameter). Set `MELODIUM_API_TOKEN` in the environment and run with `--api-report`; see Cadence.CI to obtain a token and follow execution. Nothing else: `repo_url` defaults to a real, public repository (see below). Verified end to end against real Cadence.CI infrastructure: a genuine `git clone` and `cargo build --release` of `hexyl`, a genuine `cargo test` against a genuine Postgres sidecar, and a genuine `artifact.tar.gz` written to disk.

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
- **Stage 2: test**: `rust:1.90-slim` + a `postgres:16` *service container*; a `psql` connectivity check against `DATABASE_URL` proves the sidecar is actually reachable, then `cargo test` runs the repo's own test suite. `hexyl`'s own tests are database-independent, so the `psql` check, not `cargo test`, is what this stage actually demonstrates about `service_containers`; point `repo_url` at a project with database-backed integration tests to exercise the sidecar more fully.
- **Stage 3: package**: `debian:bookworm-slim`; bundles stage 1's binary into a `.tar.gz`, streamed back and written to disk.

Stages 1 and 2 run **concurrently**; stage 3 waits for both to finish and only proceeds if neither failed.

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `dispatcher` | `CicdDispatchEngine` | Spawns each stage's container, on Mélodium Services or a local compose setup |

### Data flow

```
startup ──┬──▶ build (rust container)  ──▶ [success? true/false] ──┐
          └──▶ test  (rust + postgres) ──▶ [success? true/false] ──┴──▶ and() ──▶ filterBlock ──▶ package (debian container) ──▶ writeLocal
```

## Runtime behaviour

1. `build` and `test` both start on `startup.trigger`: two independent containers, provisioned and run in parallel, each reporting `started`/`success`/`error`/`failed`/`finished` independently. Both are explicitly pinned to the same `arch` (`|arm64()`): a service container's `arch` has no "unspecified" option the way a step's own does, so the two must agree explicitly or dispatch is rejected outright, before either container starts.
2. Two `one<void>()` treatments fan a "did *anything* fail" signal in from four separate failure/error outputs (`std/flow::one` emits the first of two incoming signals to arrive, essentially an OR-gate for `Block<void>`), driving a single abort log line without duplicating the check four times.
3. `commands` runs each entry as a direct exec, never through a shell: `${VAR}`, `&&`, `cd`, and redirection are only interpreted when the command itself is `sh -c "..."`, which is what every multi-part step here uses.
4. Each stage's three mutually exclusive outcomes (`success`/`error`/`failed`) are turned into one guaranteed `Block<bool>` (two chained `one<bool>()` per stage, `true` from `success`, `false` from either `error` or `failed`), then combined with `and()`. Stage 3 is gated on that combined `Block<bool>` via `filterBlock`, so it only ever dispatches when *both* stages truly succeeded. `flock<T>()` is not used for this: it waits for both of its `Block` inputs to resolve (value or empty close) and forwards whichever one(s) actually carried a value, so feeding it an optional signal like `success` directly would let one stage's success alone leak through even after the other failed.
5. `build`'s `data` output (the compiled binary, read from `/mnt/data/binary` inside the container) is wired directly into `package`'s `data` input: one container's filesystem output becomes another container's filesystem input, without ever touching the machine running this program in between.

### Key Mélodium patterns used

- **`simpleStep` / `simpleStepWithInput`**: the difference is exactly what it sounds like, whether the step needs data streamed *in* (as a file at `/mnt/data/<in_file>`) in addition to whatever it streams *out*.
- **`service_containers`**: sidecar containers (like the Postgres instance for stage 2) that live alongside the step's main container for its duration; `|service_container(...)`'s parameters follow its documented order (`name, memory, cpu, storage, arch, mounts, image, pull_secret, env, command`).
- **`one<void>()` as an OR-gate**: fans multiple `Block<void>` failure signals into one, without hand-writing the check four times.
- **A guaranteed `Block<bool>` plus `and()`, for a true AND-gate on an optional signal.** `success`/`error`/`failed` are mutually exclusive but only one is guaranteed to actually fire depending on what happened; folding the three into one always-present `true`/`false` value first is what makes a plain `and()` behave correctly afterward.
- **`commands` is not a shell, no matter how shell-like the strings look.** `|raw_commands` tokenises a string; it does not interpret `${VAR}`, `&&`, `cd`, or redirection. Anything that needs those needs an explicit `|command("sh", ["-c", "..."])`.
- **A required field with no "unspecified" option has to match something, deliberately.** `service_containers`' `arch` cannot be left to the scheduler the way a step's own can; the two must be pinned to the same value.
- **A showcase's default parameter is itself part of what must be verified.** A `repo_url` with no default, or one pointing at a placeholder like `my-org/my-project`, quietly makes the example unrunnable without the reader doing their own setup work first. Giving it a real, verified default is what makes `--api-report` and a token the only two things a reader needs to supply.
- **Prove infrastructure is wired correctly independently of the workload that happens to run on it**: `cargo test` here would pass identically whether or not `DATABASE_URL` pointed at a real, reachable database, so it proves nothing about the `service_containers` wiring by itself. The `psql -c "SELECT 1;"` step is a minimal, targeted check of exactly the thing this stage exists to demonstrate.

Back to the [examples index](../../README.md).
