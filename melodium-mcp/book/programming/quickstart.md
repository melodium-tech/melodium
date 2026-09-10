# First Program

This page walks through the smallest complete Mélodium program, introducing the key syntax before the full concept chapters.

## The program

The program starts up, emits a greeting string, and logs it:

```mel
use std/engine/util::startup
use std/flow::emit
use std/engine/log::logInfos

treatment main()
{
    startup()
    greet: emit<string>(value="Hello from Mélodium!")
    logInfos(label="hello")

    startup.trigger -> greet.trigger,emit -> logInfos.data
}
```

_Reference for [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [logInfos](https://doc.melodium.tech/latest/en/std/engine/log/logInfos.html)_

## What each part does

**`use` declarations** import treatments from packages. The three imports here come from the `std` package: `startup` triggers at program launch, `emit` produces a value on demand, and `logInfos` prints a stream of strings to the engine log.

**`treatment main()`** is the entry point. It has no inputs or outputs because it is the root of the program: data originates here rather than flowing in from outside.

**The treatment body** declares three inner treatment instances and connects them:

- `startup()` fires a `Block<void>` trigger when the program is ready to run.
- `greet: emit<string>(value="Hello from Mélodium!")` is an instance of `emit`, labeled `greet`, configured to emit that specific string when triggered.
- `logInfos(label="hello")` receives a `Stream<string>` and prints each value to the log under the label `hello`.

**The connection line** wires them together:

```
startup.trigger -> greet.trigger,emit -> logInfos.data
```

Read this left to right: `startup.trigger` feeds into `greet.trigger`, which starts the emit; `greet.emit` feeds into `logInfos.data`, which prints the string. The `,` shorthand chains the output of one treatment directly to the input of the next.

**`Self`** is not needed here because `main` has no declared inputs or outputs of its own. In treatments that do, `Self.input_name` and `Self.output_name` refer to those ports.

## Project layout

To run this as a project, create the following structure:

```
hello/
├── Compo.toml
└── main.mel
```

`Compo.toml`:

```toml
name    = "hello"
version = "0.1.0"

[dependencies]
std = "0.10.1"

[entrypoints]
main = "hello/main::main"
```

`main.mel` contains the treatment shown above.

## Running it

```sh
melodium run Compo.toml
```

Or as a standalone file without a project, add the metadata header to `main.mel`:

```mel
#!/usr/bin/env melodium
#! name = hello
#! version = 0.1.0
#! require = std:0.10.*

use std/engine/util::startup
use std/flow::emit
use std/engine/log::logInfos

treatment main()
{
    startup()
    greet: emit<string>(value="Hello from Mélodium!")
    logInfos(label="hello")

    startup.trigger -> greet.trigger,emit -> logInfos.data
}
```

Then run directly:

```sh
melodium main.mel
```

The engine log will output the greeting under the `hello` label. From here, the Concepts chapter explains the mental model behind treatments, connections, and tracks in full.
