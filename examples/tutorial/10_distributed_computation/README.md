# 10: Distributed Computation

**Concepts introduced:** the `distrib` package: running a treatment on a separate Mélodium engine and wiring its inputs/outputs across the network as if it were local.

This closes the tutorial track. Every example from 01 onward ran in a single process; `distribute` is the one primitive that spreads a computation across several: the mechanism the [showcase](../../showcase/) examples lean on for scaling further.

> **Note on verification:** this example needs two Mélodium engines running at once (a listener started with `melodium dist`, and this program connecting to it). It was type-checked with `melodium check` (the port names, types, and wiring follow the verified patterns already used successfully elsewhere in this tutorial), but a live two-process run was not completed in the environment that wrote it (see the setup steps below for what to try). Treat the wiring as trustworthy, but verify the live connection yourself the first time you run it.

## What it does

A remote treatment, `double`, is defined in `worker.mel`. `main.mel` connects to a running `melodium dist` engine, sends a small stream of numbers to `double`, and logs whatever comes back.

### Setup: two Mélodium engines

```
# 1. Start a listening engine, from *this project's directory* (so it can
#    resolve "distributed_computation/worker::double"):
melodium dist --localhost --port 6789 --recv-key <uuid-A> --send-key <uuid-B>

# 2. Run this program, with the keys swapped: this side sends what the
#    other side expects to receive, and vice versa:
melodium run Compo.toml --port 6789 --send_key <uuid-A> --recv_key <uuid-B>
```

`--localhost` uses an embedded certificate meant for local testing. Generate the two UUIDs however is convenient, e.g. `python3 -c "import uuid; print(uuid.uuid4())"`.

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `distributor` | `DistributionEngine` | Identifies the remote treatment (`distributed_computation/worker::double`) and its version |

### Data flow

```
main (this engine)                          worker.mel (remote engine)
──────────────────                          ──────────────────────────
generate ──▶ dispatchDouble ──send──▶ ...  ──▶  double (n -> n*2)  ──▶ ... ──recv──▶ dispatchDouble ──▶ log
```

## Runtime behaviour

1. `work/access::|new_access` builds an `Access` value (IP, port, and the two authentication keys) entirely from parameters: no cloud service involved, just a second Mélodium process reachable over the network.
2. `distrib::start` opens the connection; only once `distribStart.ready` fires does `run` actually build and send any data: nothing races the connection setup.
3. `dispatchDouble` is the general shape for "run this like a local treatment, but remotely": `distribute` allocates a `distribution_id` for one exchange, then `sendStream`/`recvStream` (tagged with matching `name`s) carry the actual data in both directions.

### Key Mélodium patterns used

- **`distribute` + `sendStream`/`recvStream`**: the three-step handshake for one remote call, allocate an ID, send input(s), receive output(s), all tagged by port name (`"n"` here) so multiple streams can cross the same connection unambiguously.
- **A model that names a treatment, not a resource**: unlike `SqlPool` or `HttpServer`, `DistributionEngine`'s parameters (`treatment`, `version`) identify *what code to run remotely*, not a resource to connect to; the network target itself comes from the `Access` value passed to `start`.
- **Keys are swapped, not shared**: the listener's `--recv-key` is the client's `send_key`, and vice versa; each side authenticates itself with the key the other side expects to receive.

Back to the [examples index](../../README.md) for the showcase track, which builds on this to distribute AI workloads and CI/CD pipelines.
