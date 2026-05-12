# Demo Script — CI Pipeline

**Duration:** ~4 min  
**Angle:** "You describe a pipeline in a few lines. Mélodium provisions the containers, runs the stages in parallel, and hands you a binary — without a CI server, without YAML, without Docker installed locally."

---

## Setup (before recording)

- Terminal split: left = editor showing `main.mel`, right = blank shell.
- Font size large enough to read both panes.
- Have `api_token` and a small public Rust repo URL ready (something with tests and a Postgres integration test, or fake one).

---

## Beat 1 — Show the code (45 s)

Open [main.mel](main.mel) in the editor. Scroll slowly from top to bottom while narrating:

> "This is the entire pipeline. Three stages. Let's walk through it."

Point to the `model Dispatcher` block:

> "One model. It connects to the Mélodium cloud API with a token. That's the only infrastructure config in the whole file."

Point to the `main` treatment, specifically lines `build` and `test`:

> "Stages 1 and 2 are triggered from the same `startup.trigger` — they start at the same time, in parallel, each in its own container."

Point to the `flock` + `bothTrigger` lines:

> "This is the fork-join. `flock` waits for both to finish, then fires stage 3."

Point to `build.data -> package.data`:

> "The binary produced by the build container flows directly into the package container — no temp file, no local disk write."

Scroll to the `test` treatment, point to `service_containers`:

> "Stage 2 requests a Postgres sidecar. The database is live inside the runner network — `DATABASE_URL` just points at it by hostname."

---

## Beat 2 — Show the structure (20 s)

Switch to the shell. Run:

```
melodium info Compo.toml
```

> "One entrypoint. Three parameters — a token, a repo URL, and an optional output path."

---

## Beat 3 — Run it (2 min 30 s)

```
melodium run Compo.toml -- \
  --api_token "$MEL_TOKEN" \
  --repo_url  "https://github.com/my-org/my-project.git"
```

**Call out each log line as it appears:**

- `[ci] pipeline started` — "Pipeline starts."
- `[build] Dispatch requested` / `[test] Dispatch requested` — "Both containers provisioning simultaneously — watch the timestamps, they're the same."
- `[build] started` + `[test] started` — "Both are running in parallel now."
- `[test]` cargo test output streaming in — "Integration tests running against a live Postgres, streamed back in real time."
- `[build] build succeeded` — "Binary compiled."
- `[test] all tests passed`
- `[package] Dispatch requested` — "Now stage 3 fires, because both are done."
- `[package] started`
- `[ci] pipeline complete — artifact written`

Then:

```
ls -lh artifact.tar.gz
tar -tzf artifact.tar.gz
```

> "The archive is here locally. It came from a container that never existed on this machine."

---

## Beat 4 — Close (15 s)

> "No CI server. No Docker. No YAML. The pipeline is the program — and it's 250 lines of readable dataflow code."

Show [main.mel](main.mel) one last time, full file visible.
