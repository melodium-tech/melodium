# 10: Distributed Computation

**Concepts introduced:** the `distrib` package: running a treatment on a separate Mélodium engine and wiring its inputs/outputs across the network as if it were local.

This closes the tutorial track. Every example from 01 onward ran in a single process; `distribute` is the one primitive that spreads a computation across several: the mechanism the [showcase](../../showcase/) examples lean on for scaling further.

## What it does

A local client sends a stream of numbers to a second, separately running Mélodium engine; that engine doubles each number and sends it back.

This needs two terminals, each running its own engine, sharing the same distribution group and a matching pair of keys (one side's send key is the other side's recv key, swapped). The UUIDs below are example values for local pairing only — any matching pair works, as long as both terminals agree:

```
# Terminal 1: the listening engine
export MELODIUM_GROUP_ID=10101010-1010-1010-1010-101010101010
melodium dist --localhost --port 6789 \
  --recv-key 11111111-1111-1111-1111-111111111111 \
  --send-key 22222222-2222-2222-2222-222222222222

# Terminal 2: this script, with the keys swapped
export MELODIUM_GROUP_ID=10101010-1010-1010-1010-101010101010
melodium run distributed_computation.mel --port 6789 \
  --send_key 11111111-1111-1111-1111-111111111111 \
  --recv_key 22222222-2222-2222-2222-222222222222
```

```
info: distrib: connected to remote engine
info: doubled: 6
info: doubled: 6
info: doubled: 6
info: doubled: 6
info: doubled: 6
```

`--localhost` uses an embedded certificate meant for local testing.

*Optional: add `--api-report` and a Mélodium Services API token (`MELODIUM_API_TOKEN`) to see this run's full trace on [Cadence.CI](https://cadence.ci/).*

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
4. With the defaults (`--amount 5 --value 3`), the client logs `connected to remote engine` once, then `doubled: 6` five times: `double` executed on the other engine, not locally.

### Key Mélodium patterns used

- **`distribute` + `sendStream`/`recvStream`**: the three-step handshake for one remote call, allocate an ID, send input(s), receive output(s), all tagged by port name (`"n"` here) so multiple streams can cross the same connection unambiguously.
- **A model that names a treatment, not a resource**: unlike `SqlPool` or `HttpServer`, `DistributionEngine`'s parameters (`treatment`, `version`) identify *what code to run remotely*, not a resource to connect to; the network target itself comes from the `Access` value passed to `start`.
- **Keys are swapped, not shared**: the listener's `--recv-key` is the client's `send_key`, and vice versa; each side authenticates itself with the key the other side expects to receive. `work/access::|new_access`'s own parameter order is `(ip, port, remote_key, self_key)`: `remote_key` is the identity presented outward (the local `send_key`), `self_key` is what is checked against what comes back (the local `recv_key`).

Back to the [examples index](../../README.md) for the showcase track, which builds on this to distribute AI workloads and CI/CD pipelines.
