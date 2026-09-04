# CI Pipeline: showcase

Not a tutorial step: a three-stage CI pipeline that runs entirely on provisioned cloud containers, combining `cicd`, `work`, and `process` in one realistic scenario.

> **Requirements:** a Mélodium Services API token, or a local `podman`/`docker compose` setup (see `CicdDispatchEngine`'s `location` parameter). Set `MELODIUM_API_TOKEN` in the environment and run with `--api-report`; see Cadence.CI to obtain a token and follow execution. Nothing else: `repo_url` defaults to a real, public repository (see below), so this is the only showcase example that runs end to end with just an API token and no separate setup. The pipeline itself is type-checked with `melodium check`; running it against live Mélodium Services infrastructure was not part of this tutorial (no token was available in the environment that wrote it), but every assumption it makes about the repository it builds was verified locally beforehand (see *How it was checked* below).

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
startup ──┬──▶ build (rust container)  ──┐
          └──▶ test  (rust + postgres) ──┴──▶ flock (wait for both) ──▶ package (debian container) ──▶ writeLocal
```

## Runtime behaviour

1. `build` and `test` both start on `startup.trigger`: two independent containers, provisioned and run in parallel, each reporting `started`/`success`/`error`/`failed`/`finished` independently.
2. Two `one<void>()` treatments fan a "did *anything* fail" signal in from four separate failure/error outputs (`std/flow::one` emits the first of two incoming signals to arrive, essentially an OR-gate for `Block<void>`), driving a single abort log line without duplicating the check four times.
3. `flock<void>()` merges `build.finished` and `test.finished` into a stream that only completes once *both* blocks have arrived; converting that back to a `Block<void>` with `trigger.start` is what actually gates stage 3.
4. `build`'s `data` output (the compiled binary, read from `/mnt/data/binary` inside the container) is wired directly into `package`'s `data` input: one container's filesystem output becomes another container's filesystem input, without ever touching the machine running this program in between.

### How it was checked

No Mélodium Services token was available while writing this, so the pipeline's own container execution was not run live. What was run live, locally, without any Mélodium involvement, was every assumption stage 1 and stage 2 make about the default repository:

```
git clone --depth 1 https://github.com/sharkdp/hexyl.git
cd hexyl
cargo build --release   # binary lands at target/release/hexyl, confirming
                         # the `grep '^name' Cargo.toml` extraction used in
                         # stage 1 finds the right name
cargo test --release    # 41 passed; 0 failed, confirming the repo's own
                         # test suite is self-contained and needs no database
```

Both commands succeeded exactly as the pipeline assumes. This is why the README above is explicit that `hexyl`'s tests do not touch Postgres: that was verified, not guessed, and it is why stage 2 adds its own `psql` connectivity check rather than relying on the demo repository's tests to prove the sidecar works.

### Key Mélodium patterns used

- **`simpleStep` / `simpleStepWithInput`**: the difference is exactly what it sounds like, whether the step needs data streamed *in* (as a file at `/mnt/data/<in_file>`) in addition to whatever it streams *out*.
- **`service_containers`**: sidecar containers (like the Postgres instance for stage 2) that live alongside the step's main container for its duration; `|service_container(...)`'s parameters follow its documented order (`name, memory, cpu, storage, arch, mounts, image, pull_secret, env, command`), confirmed against the same order used successfully elsewhere in this project rather than assumed from field listing order (see the tutorial's note on function argument order in [09_process_pipeline](../../tutorial/09_process_pipeline/)).
- **`one<void>()` as an OR-gate, `flock<void>()` as an AND-gate**: two small, reusable shapes for combining multiple `Block<void>` signals without hand-writing the logic each time.
- **A showcase's default parameter is itself part of what must be verified.** A `repo_url` with no default, or one pointing at a placeholder like `my-org/my-project`, quietly makes the example unrunnable without the reader doing their own setup work first. Giving it a real, verified default is what makes `--api-report` and a token the only two things a reader needs to supply.
- **Prove infrastructure is wired correctly independently of the workload that happens to run on it**: `cargo test` here would pass identically whether or not `DATABASE_URL` pointed at a real, reachable database, so it proves nothing about the `service_containers` wiring by itself. The `psql -c "SELECT 1;"` step is a minimal, targeted check of exactly the thing this stage exists to demonstrate.

Back to the [examples index](../../README.md).
