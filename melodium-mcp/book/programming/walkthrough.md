# Walkthrough

This page builds a complete program from scratch, introducing each concept as it is needed. The program reads a text file, trims whitespace from each line, and writes the result to another file.

It uses only the `std` and `fs` packages and is fully runnable.

## Step 1: the project files

Create this structure:

```
trimmer/
├── Compo.toml
└── main.mel
```

`Compo.toml`:

```toml
name    = "trimmer"
version = "0.1.0"

[dependencies]
std = "0.10.1"
fs  = "0.10.1"

[entrypoints]
main = "trimmer/main::main"
```

The `main` entrypoint points to the `main` treatment in `main.mel`.

## Step 2: the entry point

Start with a treatment that accepts two file paths as parameters:

```mel
use std/engine/util::startup

treatment main(const input: string = "input.txt", const output: string = "output.txt")
{
    startup()

    // More to come here.

    startup.trigger -> /* ... */
}
```

`startup()` is a treatment backed by the built-in `Engine` model. It fires a `Block<void>` trigger when the program is ready to run. Because `input` and `output` are `const` parameters, they are fixed for the entire execution and can be passed on the command line:

```sh
melodium run Compo.toml --input data.txt --output result.txt
```

## Step 3: reading the file

Add `readTextLocal`, which reads a file from the local filesystem and streams its content as a `Stream<string>`:

```mel
use std/engine/util::startup
use fs/local::readTextLocal

treatment main(const input: string = "input.txt", const output: string = "output.txt")
{
    startup()
    readTextLocal(path=input)

    startup.trigger -> readTextLocal.trigger
}
```

`readTextLocal` waits for a `Block<void>` trigger before opening the file, then streams text through its `text` output. It also exposes `completed`, `failed`, and `errors` outputs for error handling.

## Step 4: transforming each line

Add `trim` to strip leading and trailing whitespace from each string in the stream:

```mel
use std/engine/util::startup
use std/text/compose::trim
use fs/local::readTextLocal

treatment main(const input: string = "input.txt", const output: string = "output.txt")
{
    startup()
    readTextLocal(path=input)
    trim()

    startup.trigger -> readTextLocal.trigger
    readTextLocal.text -> trim.text
}
```

`trim` receives a `Stream<string>` and outputs `Stream<string>` (as `trimmed`). Every line coming out of `readTextLocal` flows through `trim` before going anywhere else.

## Step 5: writing the result

Add `writeTextLocal` to write the processed stream to the output file:

```mel
use std/engine/util::startup
use std/text/compose::trim
use fs/local::readTextLocal
use fs/local::writeTextLocal

treatment main(const input: string = "input.txt", const output: string = "output.txt")
{
    startup()
    readTextLocal(path=input)
    trim()
    writeTextLocal(path=output)

    startup.trigger -> readTextLocal.trigger
    readTextLocal.text -> trim.text,trimmed -> writeTextLocal.text
}
```

The `,` shorthand on `trim.text,trimmed` chains the `text` input and `trimmed` output of the same treatment instance, keeping the connection line compact.

## Step 6: handling errors

`readTextLocal` and `writeTextLocal` both expose `errors: Stream<string>` and `failed: Block<void>`. Connect them to the log so failures are visible:

```mel
use std/engine/util::startup
use std/text/compose::trim
use fs/local::readTextLocal
use fs/local::writeTextLocal
use std/engine/log::logErrors
use std/engine/log::logErrorMessage

treatment main(const input: string = "input.txt", const output: string = "output.txt")
{
    startup()
    readTextLocal(path=input)
    trim()
    writeTextLocal(path=output)

    readErrors:  logErrors(label="read")
    writeErrors: logErrors(label="write")
    readFail:    logErrorMessage(label="read",  message="failed to read input file")
    writeFail:   logErrorMessage(label="write", message="failed to write output file")

    startup.trigger -> readTextLocal.trigger
    readTextLocal.text -> trim.text,trimmed -> writeTextLocal.text

    readTextLocal.errors  -> readErrors.data
    readTextLocal.failed  -> readFail.trigger
    writeTextLocal.errors -> writeErrors.data
    writeTextLocal.failed -> writeFail.trigger
}
```

Nothing else changes: the main data path is unaffected. Error paths are simply additional branches from the same treatment instances.

## The complete program

```mel
use std/engine/util::startup
use std/text/compose::trim
use fs/local::readTextLocal
use fs/local::writeTextLocal
use std/engine/log::logErrors
use std/engine/log::logErrorMessage

treatment main(const input: string = "input.txt", const output: string = "output.txt")
{
    startup()
    readTextLocal(path=input)
    trim()
    writeTextLocal(path=output)

    readErrors:  logErrors(label="read")
    writeErrors: logErrors(label="write")
    readFail:    logErrorMessage(label="read",  message="failed to read input file")
    writeFail:   logErrorMessage(label="write", message="failed to write output file")

    startup.trigger -> readTextLocal.trigger
    readTextLocal.text -> trim.text,trimmed -> writeTextLocal.text

    readTextLocal.errors  -> readErrors.data
    readTextLocal.failed  -> readFail.trigger
    writeTextLocal.errors -> writeErrors.data
    writeTextLocal.failed -> writeFail.trigger
}
```

_Reference for [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [readTextLocal](https://doc.melodium.tech/latest/en/fs/local/readTextLocal.html), [trim](https://doc.melodium.tech/latest/en/std/text/compose/trim.html), [writeTextLocal](https://doc.melodium.tech/latest/en/fs/local/writeTextLocal.html)_

## Running it

```sh
melodium run Compo.toml
```

With custom paths:

```sh
melodium run Compo.toml --input data.txt --output result.txt
```

## What this demonstrates

- **`const` parameters** become CLI arguments automatically.
- **`startup()`** is how a program begins: every graph needs an event source.
- **Treatment instances** can be labeled (`readErrors:`, `writeFail:`) to distinguish multiple uses of the same treatment.
- **Error outputs** are just additional connection points, not a separate mechanism.
- **Declaration order** does not matter: `startup` is declared first, but the connections define the actual execution order.
