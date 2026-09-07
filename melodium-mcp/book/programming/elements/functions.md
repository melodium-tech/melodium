# Functions

Functions are directly callable elements, as in any other programming language, they take parameters and return a value.

Functions in Mélodium are always pure, meaning they don't have any side effects.

Functions are executed at program initialization or track creation, depending on whether their return value is used as a _constant_ or _variable_ parameter.

Functions are recognizable by the `|` symbol starting their name.

When used as a `const` parameter, the function is called once at startup and its result is shared across all tracks:

```mel
use http/status::|ok
use http/status::HttpStatus

// |ok() is evaluated once at program initialization.
model MyServer(): HttpServer {
    default_status = |ok()
}
```

When used as a `var` parameter, the function is called once per track creation, so each track may receive a different value:

```mel
use std/data/string_map::|map

treatment myHandler()
  // |map([]) is evaluated fresh for each new track.
  var headers: StringMap = |map([])
{
    // …
}
```

_Reference for [|ok](https://doc.melodium.tech/latest/en/http/status/|ok.html), [|map](https://doc.melodium.tech/latest/en/std/data/string_map/|map.html)_
