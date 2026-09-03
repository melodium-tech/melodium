# CI Pipeline: showcase

Not a tutorial step: a three-stage CI pipeline that runs entirely on provisioned cloud containers, combining `cicd`, `work`, and `process` in one realistic scenario.

> **Requirements:** a Mélodium Services API token, or a local `podman`/`docker compose` setup (see `CicdDispatchEngine`'s `location` parameter). Set `MELODIUM_API_TOKEN` in the environment and run with `--api-report`; see Cadence.CI to obtain a token and follow execution. This example is type-checked with `melodium check` but was not run against live infrastructure for this tutorial.

## What it does

```
export MELODIUM_API_TOKEN="my-api-token"
melodium run --api-report Compo.toml \
  --repo_url "https://github.com/my-org/my-project.git"
```

- **Stage 1: build**: `rust:1.82-slim` container; `cargo build --release`; the compiled binary streams back locally.
- **Stage 2: test**: `rust:1.82-slim` + a `postgres:16` *service container*; `cargo test`, with `DATABASE_URL` wired to the sidecar.
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

### Key Mélodium patterns used

- **`simpleStep` / `simpleStepWithInput`**: the difference is exactly what it sounds like, whether the step needs data streamed *in* (as a file at `/mnt/data/<in_file>`) in addition to whatever it streams *out*.
- **`service_containers`**: sidecar containers (like the Postgres instance for stage 2) that live alongside the step's main container for its duration; `|service_container(...)`'s parameters follow its documented order (`name, memory, cpu, storage, arch, mounts, image, pull_secret, env, command`), confirmed against the same order used successfully elsewhere in this project rather than assumed from field listing order (see the tutorial's note on function argument order in [09_process_pipeline](../../tutorial/09_process_pipeline/)).
- **`one<void>()` as an OR-gate, `flock<void>()` as an AND-gate**: two small, reusable shapes for combining multiple `Block<void>` signals without hand-writing the logic each time.

Back to the [examples index](../../README.md).
