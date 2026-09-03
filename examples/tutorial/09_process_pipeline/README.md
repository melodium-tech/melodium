# 09: Process Pipeline

**Concepts introduced:** running an external command as a treatment (`process`), streaming data through a subprocess's stdin/stdout.

Reads a file, pipes its content through the operating system's `sort` command, and writes the sorted result. Mélodium orchestrates the subprocess and its I/O streams; the actual sorting is done entirely by `sort` itself.

## What it does

```
melodium run Compo.toml --input_file fruits.txt
```

Turns `banana / apple / cherry / date / elderberry` into an alphabetically sorted `sorted.txt`.

## How it is built

No models: `process/local::spawnOnce` is a convenience treatment that obtains a local executor on its own.

### Data flow

```
readLocal ──▶ proc.stdin
proc.stdout ──▶ decode ──▶ drop blank chunk ──▶ writeTextLocal
proc.stderr ──▶ decode ──▶ drop blank chunk ──▶ logErrors
proc.exit   ──▶ unwrapOr ──▶ logDataInfo
```

## Runtime behaviour

1. `read.data` (the file's bytes) is wired straight into `proc.stdin`; `sort` starts reading as soon as bytes arrive, before the file has even finished being read: there is no "read the whole file, then start the subprocess" step.
2. `proc.exit` is a `Block<Option<i32>>` (`none` if the process was killed by a signal rather than exiting normally); `std/ops/option/block::unwrapOr`, the *block* variant and not the stream one used elsewhere in this tutorial, supplies a fallback.
3. Both `stdout` and `stderr` are decoded and filtered for the trailing-empty-chunk gotcha from example 05: without it, a perfectly successful run would still log a spurious empty "error" line from stderr.

### Key Mélodium patterns used

- **A subprocess is just another treatment with stream ports**: `stdin`/`stdout`/`stderr` are `Stream<byte>` like any HTTP body or file content; every idiom from earlier examples (decode, filter blanks, encode) applies unchanged.
- **Block vs. stream variants of the same helper**: `std/ops/option::unwrapOr` (stream) and `std/ops/option/block::unwrapOr` (block) do the same job on different port kinds; picking the wrong one is a type error `melodium check` catches immediately.
- **`|command(name, arguments)`**: builds a `Command` value from a plain executable name and an argument vector (empty here); function argument order was confirmed by testing rather than assumed, since it does not always match a function's documented parameter order.

Next: [10_distributed_computation](../10_distributed_computation/) closes out the tutorial track with Mélodium's distributed dataflow model.
