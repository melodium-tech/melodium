# 10: Distributed Computation

**Concepts introduced:** the `distrib` package: running a treatment on a separate Mélodium engine and wiring its inputs/outputs across the network as if it were local.

This closes the tutorial track. Every example from 01 onward ran in a single process; `distribute` is the one primitive that spreads a computation across several: the mechanism the [showcase](../../showcase/) examples lean on for scaling further.

Verified live: two real `melodium` processes, one listening, one connecting, with a real 5-item stream sent and doubled by the remote side.

## What it does

This is a single standalone script, `distributed_computation.mel` (no `Compo.toml`), unlike every other example in this tutorial. A `DistributionEngine` resolves its target treatment against the local engine's own compiled collection, and a `Compo.toml` project only compiles what is reachable from the entrypoint being run: `double` is never directly instantiated anywhere in `main`'s own graph, it is only referenced by name in `Doubler`'s configuration, so a project build prunes it and `DistributionEngine` can never find it at runtime. A standalone script compiles as a single unit with nothing pruned, so `double` stays available. Point a `DistributionEngine` at a treatment you *do* also instantiate locally somewhere and this does not apply.

### Setup: two Mélodium engines

Both engines must share the same distribution group, or they can never pair up (each defaults to its own random one):

```
export MELODIUM_GROUP_ID=$(python3 -c "import uuid; print(uuid.uuid4())")
```

Run in the same shell (or `export` the same value into two separate ones) before starting each engine below.

```
# 1. Start a listening engine:
melodium dist --localhost --port 6789 --recv-key <uuid-A> --send-key <uuid-B>

# 2. Run this script, with the keys swapped: this side sends what the
#    other side expects to receive, and vice versa:
melodium run distributed_computation.mel --port 6789 --send_key <uuid-A> --recv_key <uuid-B>
```

`--localhost` uses an embedded certificate meant for local testing. Generate `<uuid-A>`/`<uuid-B>` the same way as the group ID above.

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `distributor` | `DistributionEngine` | Identifies the remote treatment (`distributed_computation::double`) and its version |

### Data flow

```
main (this engine)                          double (remote engine)
──────────────────                          ──────────────────────
generate ──▶ dispatchDouble ──send──▶ ...  ──▶  double (n -> n*2)  ──▶ ... ──recv──▶ dispatchDouble ──▶ log
```

### Reference

- **`distrib`**: [DistributionEngine](https://doc.melodium.tech/latest/en/distrib/DistributionEngine.html), [start](https://doc.melodium.tech/latest/en/distrib/start.html), [distribute](https://doc.melodium.tech/latest/en/distrib/distribute.html), [sendStream](https://doc.melodium.tech/latest/en/distrib/sendStream.html), [recvStream](https://doc.melodium.tech/latest/en/distrib/recvStream.html)
- **`net`**: [|localhost_ipv4](https://doc.melodium.tech/latest/en/net/ip/|localhost_ipv4.html), [|from_ipv4](https://doc.melodium.tech/latest/en/net/ip/|from_ipv4.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logInfos](https://doc.melodium.tech/latest/en/std/engine/log/logInfos.html), [logError](https://doc.melodium.tech/latest/en/std/engine/log/logError.html), [logErrorMessage](https://doc.melodium.tech/latest/en/std/engine/log/logErrorMessage.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [generate](https://doc.melodium.tech/latest/en/std/flow/generate.html), [trigger](https://doc.melodium.tech/latest/en/std/flow/trigger.html), [toString](https://doc.melodium.tech/latest/en/std/conv/toString.html), [|map](https://doc.melodium.tech/latest/en/std/data/map/|map.html), [add](https://doc.melodium.tech/latest/en/std/ops/num/add.html)
- **`work`**: [Access](https://doc.melodium.tech/latest/en/work/access/Access.html), [|new_access](https://doc.melodium.tech/latest/en/work/access/|new_access.html)

## Runtime behaviour

1. `work/access::|new_access` builds an `Access` value (IP, port, and the two authentication keys) entirely from parameters: no cloud service involved, just a second Mélodium process reachable over the network.
2. `distrib::start` opens the connection; only once `distribStart.ready` fires does `run` actually build and send any data: nothing races the connection setup.
3. `dispatchDouble` is the general shape for "run this like a local treatment, but remotely": `distribute` allocates a `distribution_id` for one exchange, then `sendStream`/`recvStream` (tagged with matching `name`s) carry the actual data in both directions.
4. A real run, `--amount 5 --value 3`, logs `connected to remote engine` once, then `doubled: 6` five times: `double` really executed on the other engine, not locally.

### Key Mélodium patterns used

- **`distribute` + `sendStream`/`recvStream`**: the three-step handshake for one remote call, allocate an ID, send input(s), receive output(s), all tagged by port name (`"n"` here) so multiple streams can cross the same connection unambiguously.
- **A model that names a treatment, not a resource**: unlike `SqlPool` or `HttpServer`, `DistributionEngine`'s parameters (`treatment`, `version`) identify *what code to run remotely*, not a resource to connect to; the network target itself comes from the `Access` value passed to `start`.
- **Keys are swapped, not shared**: the listener's `--recv-key` is the client's `send_key`, and vice versa; each side authenticates itself with the key the other side expects to receive. `work/access::|new_access`'s own parameter order is `(ip, port, remote_key, self_key)`: `remote_key` is the identity presented outward (the local `send_key`), `self_key` is what is checked against what comes back (the local `recv_key`).
- **A standalone script as the deliberate choice, not the default.** Every other example in this tutorial is a `Compo.toml` project; this one is a single `.mel` file specifically because that is what makes `DistributionEngine`'s target treatment resolvable, per *What it does* above.

Back to the [examples index](../../README.md) for the showcase track, which builds on this to distribute AI workloads and CI/CD pipelines.
