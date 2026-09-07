# Error Handling

Mélodium has no exceptions. Errors are data, flowing through dedicated outputs just like any other value. A treatment that can fail exposes this through its output ports rather than interrupting execution.

## The standard error pattern

Most treatments that interact with external systems (files, network, databases) expose a consistent set of outputs:

- `completed: Block<void>` - emitted when the operation finishes successfully.
- `failed: Block<void>` - emitted when the operation fails.
- `finished: Block<void>` - emitted when the operation ends, regardless of whether it succeeded or failed.
- `errors: Stream<string>` - streams error messages as they occur.

```mel
use fs/local::writeTextLocal
use std/engine/log::logErrors
use std/engine/log::logInfoMessage

treatment saveResult(path: string)
  input text: Stream<string>
{
    writeTextLocal(path=path)
    logErrors(label="save")
    logSuccess: logInfoMessage(label="save", message="file written successfully")

    Self.text -----------------------> writeTextLocal.text
    writeTextLocal.errors ---------> logErrors.data
    writeTextLocal.completed ------> logSuccess.trigger
}
```

_Reference for [writeTextLocal](https://doc.melodium.tech/latest/en/fs/local/writeTextLocal.html), [logErrors](https://doc.melodium.tech/latest/en/std/engine/log/logErrors.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html)_

Unused outputs are legal: if `failed` and `errors` are not connected, error information is silently dropped. This is valid but not recommended for production code.

## Handling failure explicitly

When the distinction between success and failure matters for the program flow, connect `failed` to a handler treatment. The `failed` output fires a `Block<void>` trigger, which can chain into other treatments:

```mel
use fs/local::readTextLocal
use std/flow::emit
use std/engine/log::logErrorMessage
use std/engine/log::logErrors

treatment loadConfig(path: string)
  input  trigger: Block<void>
  output text: Stream<string>
{
    readTextLocal(path=path)
    logErrors(label="config")
    logFail: logErrorMessage(label="config", message="could not read config file")

    Self.trigger -----------------> readTextLocal.trigger
    readTextLocal.text -----------> Self.text
    readTextLocal.errors ---------> logErrors.data
    readTextLocal.failed ---------> logFail.trigger
}
```

## Option<T> for absence

Functions and some treatments return `Option<T>` when a value may or may not be present, rather than using a separate error output. `Option<T>` is either `Some(value)` or `None`.

The `std` package provides treatments for working with options: `unwrapOr` substitutes a default when the value is `None`, and `check`/`uncheck` split a stream into present and absent paths.

```mel
use std/ops/option::unwrapOr

treatment withDefault()
  input  value: Stream<Option<string>>
  output result: Stream<string>
{
    unwrapOr<string>(default="(none)")

    Self.value -> unwrapOr.option,value -> Self.result
}
```

_Reference for [unwrapOr](https://doc.melodium.tech/latest/en/std/ops/option/unwrapOr.html)_

## Summary

| Situation | Mechanism |
|-----------|-----------|
| Operation may fail | `failed: Block<void>` output |
| Error messages | `errors: Stream<string>` output |
| Successful completion | `completed: Block<void>` output |
| Value may be absent | `Option<T>` return type |

Because errors are just data, the same connection rules apply: they can be logged, counted, transformed, or forwarded to other treatments exactly like any other stream.
