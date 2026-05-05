# Mélodium Elements in Rust — Reference

This reference catalogs every treatment, function, model, data type, and context exposed by the Mélodium standard library packages, organized by package and area. Use it when writing Mélodium code that imports from these packages.

All import paths use the Mélodium area syntax (e.g. `use std/flow::emit`). Signatures are written in Mélodium notation, not Rust.

---

## `std` — Standard Library

### `std/engine/util`

```mel
use std/engine/util::startup
```

| Element | Kind | Signature |
|---------|------|-----------|
| `startup` | treatment | input *(none)* → output `trigger Block<void>` |

The `startup` treatment is the canonical entry point trigger. It fires exactly once when the engine is ready.

---

### `std/engine/log`

```mel
use std/engine/log::logInfoMessage   // convenience wrappers
use std/engine/log::log_stream
use std/engine/log::Level
```

**`Level` data type** — log severity level.

| Function | Returns |
|----------|---------|
| `\|error() -> Level` | error level |
| `\|warning() -> Level` | warning level |
| `\|info() -> Level` | info level |
| `\|debug() -> Level` | debug level |
| `\|trace() -> Level` | trace level |

**Logging treatments** (all require `[engine: Engine]` model param, which is implicit when using the default engine):

| Treatment | Inputs | Outputs |
|-----------|--------|---------|
| `log_stream[engine](level: Level, label: string)` | `messages Stream<string>` | *(none)* |
| `log_stream_label[engine](level: Level)` | `label Block<string>`, `messages Stream<string>` | *(none)* |
| `log_block[engine](level: Level, label: string)` | `message Block<string>` | *(none)* |
| `log_block_label[engine](level: Level)` | `label Block<string>`, `message Block<string>` | *(none)* |
| `log_data_stream<D: Display>[engine](level, label)` | `display Stream<D>` | *(none)* |
| `log_data_stream_label<D: Display>[engine](level)` | `label Block<string>`, `display Stream<D>` | *(none)* |
| `log_data_block<D: Display>[engine](level, label)` | `display Block<D>` | *(none)* |
| `log_data_block_label<D: Display>[engine](level)` | `label Block<string>`, `display Block<D>` | *(none)* |

**Convenience (deprecated-style) wrappers** available at the same area:

| Treatment | Description |
|-----------|-------------|
| `logInfoMessage(message: string)` | input `trigger Block<void>` → logs at info level |

---

### `std/engine` (Engine model)

```mel
use std/engine::Engine
```

| Element | Kind | Description |
|---------|------|-------------|
| `Engine` | model | The runtime engine. Usually implicit; needed for explicit log routing. |
| `\|version() -> string` | function | Returns the Mélodium engine version string. |
| `\|run_id() -> string` | function | Returns the unique run identifier. |
| `\|group_id() -> string` | function | Returns the group identifier. |

`Engine` source `ready` → emits `trigger Block<void>` at startup (same as `startup` treatment).

---

### `std/flow`

```mel
use std/flow::emit
use std/flow::stream
use std/flow::trigger
// etc.
```

| Treatment | Parameters | Inputs | Outputs |
|-----------|-----------|--------|---------|
| `emit<T>(value: T)` | — | `trigger Block<void>` | `emit Block<T>` |
| `stream<T>` | — | `block Block<T>` | `stream Stream<T>` |
| `chain<T>` | — | `first Stream<T>`, `second Stream<T>` | `chained Stream<T>` |
| `trigger<T>` | — | `stream Stream<T>` | `start Block<void>`, `end Block<void>`, `first Block<T>`, `last Block<T>` |
| `check<T>` | — | `value Block<T>` | `check Block<void>` |
| `uncheck<T>` | — | `value Block<T>` | `uncheck Block<void>` *(emitted if never received)* |
| `merge<T>` | — | `a Stream<T>`, `b Stream<T>` | `value Stream<T>` *(non-deterministic order)* |
| `arrange<T>` | — | `a Stream<T>`, `b Stream<T>`, `select Stream<bool>` | `value Stream<T>` |
| `fill<T>(value: T)` | — | `pattern Stream<void>` | `filled Stream<T>` |
| `filter<T>` | — | `value Stream<T>`, `select Stream<bool>` | `accepted Stream<T>`, `rejected Stream<T>` |
| `filterBlock<T>` | — | `value Block<T>`, `select Block<bool>` | `accepted Block<T>`, `rejected Block<T>` |
| `fit<T>` | — | `value Stream<T>`, `pattern Stream<void>` | `fitted Stream<T>` |
| `count<T>` | — | `stream Stream<T>` | `count Stream<u128>` *(1-based)* |
| `generate<T>(data: T)` | — | `length Block<u128>` | `stream Stream<T>` |
| `generateIndefinitely<T>(data: T)` | — | `trigger Block<void>` | `stream Stream<T>` |
| `insert<T>` | — | `stream Stream<T>`, `block Block<T>` | `output Stream<T>` |
| `flock<T>` | — | `a Block<T>`, `b Block<T>` | `stream Stream<T>` |
| `one<T>` | — | `a Block<T>`, `b Block<T>` | `value Block<T>` *(first to arrive)* |
| `close<T>` | — | `trigger Block<void>` | `closed Stream<T>` *(immediately closed, no elements)* |
| `closeBlock<T>` | — | `trigger Block<void>` | `closed Block<T>` *(immediately closed)* |
| `consume<T>` | — | `stream Stream<T>` | *(none)* |
| `pass<T>(cond: bool)` | — | `stream Stream<T>` | `passed Stream<T>` |
| `passBlock<T>(cond: bool)` | — | `block Block<T>` | `passed Block<T>` |
| `barrier<T>` | — | `leverage Block<bool>`, `stream Stream<T>` | `passed Stream<T>` |
| `cut<T>` | — | `cut Block<void>`, `stream Stream<T>` | `passed Stream<T>` |
| `release<T>` | — | `leverage Block<void>`, `data Stream<T>` | `released Stream<T>` |
| `releaseBlock<T>` | — | `leverage Block<void>`, `data Block<T>` | `released Block<T>` |
| `waitBlock<T>` | — | `a Block<T>`, `b Block<T>` | `awaited Block<void>` |

---

### `std/flow/vec`

```mel
use std/flow/vec::flatten
use std/flow/vec::size
// etc.
```

| Treatment | Inputs | Outputs |
|-----------|--------|---------|
| `flatten<T>` | `vector Stream<Vec<T>>` | `value Stream<T>` |
| `pattern<T>` | `stream Stream<Vec<T>>` | `pattern Stream<Vec<void>>` |
| `fit<T>` | `value Stream<T>`, `pattern Stream<Vec<void>>` | `fitted Stream<Vec<T>>` |
| `fill<T>(value: T)` | `pattern Stream<Vec<void>>` | `filled Stream<Vec<T>>` |
| `size<T>` | `vector Stream<Vec<T>>` | `size Stream<u64>` |
| `resize<T>(default: T)` | `vector Stream<Vec<T>>`, `size Stream<u64>` | `resized Stream<Vec<T>>` |

---

### `std/flow/concentrate`

```mel
use std/flow/concentrate::Concentrator
use std/flow/concentrate::concentrate
use std/flow/concentrate::concentrated
```

| Element | Kind | Description |
|---------|------|-------------|
| `Concentrator` | model | Collects streams across multiple tracks and re-emits them together. |
| `concentrate<T>[concentrator: Concentrator]` | treatment | input `data Stream<T>` — feeds data into the concentrator |
| `concentrateBlock<T>[concentrator: Concentrator]` | treatment | input `data Block<T>` — feeds a block value into the concentrator |
| `concentrated<T>[concentrator: Concentrator]` | treatment | input `trigger Block<T>` → output `data Stream<T>` — retrieves concentrated data |

---

### `std/conv`

```mel
use std/conv::toString
use std/conv::toBytes
use std/conv::toI64
// etc.
```

All conversions come in two forms: a pure function (`|name`) and a streaming treatment (`name`). The treatment takes a `Stream<T>` input named `value` and produces a `Stream<R>` output named `into` (or `option` for fallible conversions).

**Infallible conversions** (trait-constrained):

| Mélodium name | Input type | Output type | Trait required |
|---------------|-----------|-------------|----------------|
| `toVoid<T>` / `\|toVoid<T>(v) -> void` | `T` | `void` | *(any)* |
| `toBytes<T>` / `\|toBytes<T>(v) -> Vec<byte>` | `T` | `Vec<byte>` | *(any)* |
| `toI8<T>` / `\|toI8` | `T` | `i8` | `ToI8` |
| `toI16<T>` / `\|toI16` | `T` | `i16` | `ToI16` |
| `toI32<T>` / `\|toI32` | `T` | `i32` | `ToI32` |
| `toI64<T>` / `\|toI64` | `T` | `i64` | `ToI64` |
| `toI128<T>` / `\|toI128` | `T` | `i128` | `ToI128` |
| `toU8<T>` / `\|toU8` | `T` | `u8` | `ToU8` |
| `toU16<T>` / `\|toU16` | `T` | `u16` | `ToU16` |
| `toU32<T>` / `\|toU32` | `T` | `u32` | `ToU32` |
| `toU64<T>` / `\|toU64` | `T` | `u64` | `ToU64` |
| `toU128<T>` / `\|toU128` | `T` | `u128` | `ToU128` |
| `toF32<T>` / `\|toF32` | `T` | `f32` | `ToF32` |
| `toF64<T>` / `\|toF64` | `T` | `f64` | `ToF64` |
| `toString<T>` / `\|toString` | `T` | `string` | `ToString` |

**Fallible conversions** (output is `Option<R>`):

| Mélodium name | Output type |
|---------------|-------------|
| `tryToI8<T: TryToI8>` / `\|tryToI8` | `Option<i8>` |
| `tryToI16` … `tryToI128` | `Option<iN>` |
| `tryToU8` … `tryToU128` | `Option<uN>` |
| `tryToF32`, `tryToF64` | `Option<fN>` |
| `tryToString` | `Option<string>` |

**Saturating conversions** (clamp instead of overflow):

| Mélodium name | Trait required |
|---------------|----------------|
| `saturatingToI8<T: SaturatingToI8>` | `SaturatingToI8` |
| … (same pattern for all numeric types) | … |
| `saturatingToF64` | `SaturatingToF64` |

---

### `std/ops` — Comparison

```mel
use std/ops::|equal
use std/ops::equal
```

**Functions:**

| Function | Signature |
|----------|-----------|
| `\|condition<T>(condition: bool, a: T, b: T) -> T` | If `condition` is true returns `a`, else `b` |
| `\|equal<T: PartialEquality>(a: T, b: T) -> bool` | |
| `\|notEqual<T: PartialEquality>(a: T, b: T) -> bool` | |
| `\|greaterThan<T: PartialOrder>(a: T, b: T) -> bool` | |
| `\|greaterEqual<T: PartialOrder>(a: T, b: T) -> bool` | |
| `\|lowerThan<T: PartialOrder>(a: T, b: T) -> bool` | |
| `\|lowerEqual<T: PartialOrder>(a: T, b: T) -> bool` | |
| `\|min<T: Order>(a: T, b: T) -> T` | |
| `\|max<T: Order>(a: T, b: T) -> T` | |
| `\|clamp<T: Order>(value: T, min: T, max: T) -> T` | |

**Treatments** (streaming; `a Stream<T>`, `b Stream<T>` → `result Stream<bool>` or `value Stream<T>`):

| Treatment | Output |
|-----------|--------|
| `equal<T: PartialEquality>` | `result Stream<bool>` |
| `notEqual<T: PartialEquality>` | `result Stream<bool>` |
| `greaterThan<T: PartialOrder>` | `result Stream<bool>` |
| `greaterEqual<T: PartialOrder>` | `result Stream<bool>` |
| `lowerThan<T: PartialOrder>` | `result Stream<bool>` |
| `lowerEqual<T: PartialOrder>` | `result Stream<bool>` |
| `min<T: Order>` | `value Stream<T>` |
| `max<T: Order>` | `value Stream<T>` |
| `clamp<T: Order>(min: T, max: T)` | `value Stream<T>` |

---

### `std/ops/num`

```mel
use std/ops/num::|add
use std/ops/num::add
```

All numeric operations come as both functions and streaming treatments. Functions take scalar args; treatments take matching `Stream` inputs.

| Function | Trait | Returns |
|----------|-------|---------|
| `\|abs<N: Signed>(value: N) -> Option<N>` | `Signed` | `None` if overflow |
| `\|signum<N: Signed>(value: N) -> N` | `Signed` | |
| `\|isPositive<N: Signed>(value: N) -> bool` | `Signed` | |
| `\|isNegative<N: Signed>(value: N) -> bool` | `Signed` | |
| `\|add<N: Add>(a: N, b: N) -> N` | `Add` | |
| `\|checkedAdd<N: CheckedAdd>(a, b) -> Option<N>` | `CheckedAdd` | |
| `\|saturatingAdd<N: SaturatingAdd>(a, b) -> N` | `SaturatingAdd` | |
| `\|wrappingAdd<N: WrappingAdd>(a, b) -> N` | `WrappingAdd` | |
| `\|sub<N: Sub>` | `Sub` | |
| `\|checkedSub`, `\|saturatingSub`, `\|wrappingSub` | … | |
| `\|mul<N: Mul>` | `Mul` | |
| `\|checkedMul`, `\|saturatingMul`, `\|wrappingMul` | … | |
| `\|div<N: Div>(a, b) -> N` | `Div` | |
| `\|checkedDiv` | `CheckedDiv` | `Option<N>` |
| `\|rem<N: Rem>` | `Rem` | |
| `\|checkedRem` | `CheckedRem` | `Option<N>` |
| `\|neg<N: Neg>(value: N) -> N` | `Neg` | |
| `\|checkedNeg`, `\|wrappingNeg` | … | |
| `\|pow<N: Pow>(value: N, exp: u32) -> N` | `Pow` | |
| `\|checkedPow` | `CheckedPow` | `Option<N>` |
| `\|euclidDiv<N: Euclid>(a, b) -> N` | `Euclid` | |
| `\|euclidRem<N: Euclid>(a, b) -> N` | `Euclid` | |
| `\|checkedEuclidDiv`, `\|checkedEuclidRem` | `CheckedEuclid` | `Option<N>` |

Streaming treatments: same names without `\|`, inputs `a Stream<N>` + `b Stream<N>` → `result Stream<R>`.

For unary ops (`abs`, `signum`, `isPositive`, `isNegative`, `neg`): input `value Stream<N>` → output `result Stream<R>`.

For `pow`: inputs `value Stream<N>`, `exp Stream<u32>` → `result Stream<R>`.

---

### `std/ops/float`

```mel
use std/ops/float::|sqrt
use std/ops/float::sqrt
```

**Functions** (all take `value: F` where `F: Float`):

| Function | Description |
|----------|-------------|
| `\|isNan<F: Float>(value: F) -> bool` | |
| `\|isFinite`, `\|isInfinite`, `\|isNormal`, `\|isSubnormal` | predicates |
| `\|floor`, `\|ceil`, `\|round`, `\|trunc`, `\|fract` | rounding |
| `\|recip` | 1/x |
| `\|pow<F: Float>(value: F, exp: F) -> F` | float exponent |
| `\|sqrt`, `\|cbrt` | roots |
| `\|exp`, `\|exp2` | exponentials |
| `\|ln`, `\|log2`, `\|log10` | logarithms |
| `\|log(value: F, base: F) -> F` | arbitrary base |
| `\|hypot(a: F, b: F) -> F` | |
| `\|sin`, `\|cos`, `\|tan` | trig |
| `\|asin`, `\|acos`, `\|atan` | inverse trig |
| `\|atan2(y: F, x: F) -> F` | |
| `\|sinh`, `\|cosh`, `\|tanh` | hyperbolic |
| `\|asinh`, `\|acosh`, `\|atanh` | inverse hyperbolic |
| `\|toDegrees`, `\|toRadians` | angle conversion |

All have streaming treatment equivalents with identical names (without `\|`), input `value Stream<F>` → output `result Stream<R>`.

---

### `std/ops/bin`

```mel
use std/ops/bin::|and
use std/ops/bin::and
```

| Function | Signature |
|----------|-----------|
| `\|and<B: Binary>(a: B, b: B) -> B` | bitwise AND |
| `\|or<B: Binary>(a: B, b: B) -> B` | bitwise OR |
| `\|xor<B: Binary>(a: B, b: B) -> B` | bitwise XOR |
| `\|not<B: Binary>(value: B) -> B` | bitwise NOT |

Treatments: `and`, `or`, `xor` take inputs `a Stream<B>`, `b Stream<B>` → `result Stream<B>`. `not` takes `value Stream<B>` → `result Stream<B>`.

---

### `std/ops/option` (Stream)

```mel
use std/ops/option::unwrap
use std/ops/option::unwrapOr
use std/ops/option::|wrap
```

| Element | Kind | Inputs | Outputs |
|---------|------|--------|---------|
| `unwrap<T>` | treatment | `option Stream<Option<T>>` | `value Stream<T>` *(Nones silently dropped)* |
| `unwrapOr<T>(default: T)` | treatment | `option Stream<Option<T>>` | `value Stream<T>` |
| `fuse<T>` | treatment | `option Stream<Option<T>>` | `value Stream<T>` *(stops at first None)* |
| `wrap<T>` | treatment | `value Stream<T>` | `option Stream<Option<T>>` |
| `\|unwrapOr<T>(option: Option<T>, default: T) -> T` | function | — | — |
| `\|wrap<T>(value: T) -> Option<T>` | function | — | — |

---

### `std/ops/option/block` (Block)

```mel
use std/ops/option/block::unwrap
use std/ops/option/block::map
```

| Element | Kind | Inputs | Outputs |
|---------|------|--------|---------|
| `unwrap<T>` | treatment | `option Block<Option<T>>` | `value Block<T>` |
| `unwrapOr<T>(default: T)` | treatment | `option Block<Option<T>>` | `value Block<T>` |
| `wrap<T>` | treatment | `value Block<T>` | `option Block<Option<T>>` |
| `map<T>` | treatment | `option Block<Option<T>>` | `none Block<void>`, `value Block<T>` |
| `reduce<T>` | treatment | `value Block<T>`, `none Block<void>` | `option Block<Option<T>>` |

---

### `std/ops/vec` (Stream)

```mel
use std/ops/vec::contains
use std/ops/vec::concat
use std/ops/vec::|contains
```

| Element | Kind | Inputs | Outputs |
|---------|------|--------|---------|
| `contains<T: PartialEquality>` | treatment | `value Stream<T>`, `vec Stream<Vec<T>>` | `contains Stream<bool>` |
| `concat<T>` | treatment | `first Stream<Vec<T>>`, `second Stream<Vec<T>>` | `concatened Stream<Vec<T>>` |
| `\|contains<T: PartialEquality>(vector: Vec<T>, value: T) -> bool` | function | — | — |
| `\|concat<T>(first: Vec<T>, second: Vec<T>) -> Vec<T>` | function | — | — |

---

### `std/ops/vec/block` (Block)

Same API as `std/ops/vec` but all ports are `Block<…>`.

---

### `std/data/map`

```mel
use std/data/map::Map
use std/data/map::|entry
use std/data/map::entry
```

**`Map` data type** — heterogeneous key-value map (string keys, any values).

**Functions:**

| Function | Signature |
|----------|-----------|
| `\|map(entries: Vec<Map>) -> Map` | Construct from a list of entries |
| `\|entry<T>(key: string, value: T) -> Map` | Create a single-entry map |
| `\|get<T>(map: Map, key: string) -> Option<T>` | Look up a value |
| `\|insert<T>(map: Map, key: string, value: T) -> Map` | Return map with added entry |

**Treatments (Stream):**

| Treatment | Inputs | Outputs |
|-----------|--------|---------|
| `entry<T>(key: string)` | `value Stream<T>` | `map Stream<Map>` |
| `get<T>(key: string)` | `map Stream<Map>` | `value Stream<Option<T>>` |
| `insert<T>(key: string)` | `base Stream<Map>`, `value Stream<T>` | `map Stream<Map>` |

**Treatments (Block) — `std/data/map/block`:**

Same names; all ports are `Block<…>`. Additionally:

| Treatment | Inputs | Outputs |
|-----------|--------|---------|
| `merge` | `a Block<Map>`, `b Block<Map>` | `map Block<Map>` |

---

### `std/data/string_map`

```mel
use std/data/string_map::StringMap
use std/data/string_map::|map
use std/data/string_map::|get
```

**`StringMap` data type** — string-to-string map. Used for HTTP headers, parameters, formatting entries, etc.

**Functions:**

| Function | Signature |
|----------|-----------|
| `\|map(entries: Vec<StringMap>) -> StringMap` | Construct from a list of entries |
| `\|entry(key: string, value: string) -> StringMap` | Single entry |
| `\|get(map: StringMap, key: string) -> Option<string>` | Look up |
| `\|insert(map: StringMap, key: string, value: string) -> StringMap` | Insert |

**Treatments (Stream):**

| Treatment | Inputs | Outputs |
|-----------|--------|---------|
| `entry(key: string)` | `value Stream<string>` | `map Stream<StringMap>` |
| `get(key: string)` | `map Stream<StringMap>` | `value Stream<Option<string>>` |
| `insert(key: string)` | `base Stream<StringMap>`, `value Stream<string>` | `map Stream<StringMap>` |

**Treatments (Block) — `std/data/string_map/block`:** same names, Block ports, plus `merge`.

---

### `std/text/compose`

```mel
use std/text/compose::rescale
use std/text/compose::split
use std/text/compose::|format
```

| Element | Kind | Inputs | Outputs |
|---------|------|--------|---------|
| `rescale(delimiter: string)` | treatment | `unscaled Stream<string>` | `scaled Stream<string>` |
| `split(delimiter: string, inclusive: bool)` | treatment | `text Stream<string>` | `splitted Stream<Vec<string>>` |
| `trim` | treatment | `text Stream<string>` | `trimmed Stream<string>` |
| `trimEnd` | treatment | `text Stream<string>` | `trimmed Stream<string>` |
| `trimStart` | treatment | `text Stream<string>` | `trimmed Stream<string>` |
| `format(format: string)` | treatment | `entries Stream<StringMap>` | `formatted Stream<string>` |
| `checkedFormat(format: string)` | treatment | `entries Stream<StringMap>` | `formatted Stream<Option<string>>` |
| `\|split(text: string, delimiter: string, inclusive: bool) -> Vec<string>` | function | — | — |
| `\|trim(text: string) -> string` | function | — | — |
| `\|trimEnd`, `\|trimStart` | function | — | — |
| `\|format(format: string, entries: StringMap) -> string` | function | — | — |
| `\|checkedFormat(format: string, entries: StringMap) -> Option<string>` | function | — | — |

---

### `std/text/convert/string`

```mel
use std/text/convert/string::toUtf8
use std/text/convert/string::fromUtf8
```

| Element | Kind | Input | Output |
|---------|------|-------|--------|
| `toChar` | treatment | `text Stream<string>` | `chars Stream<Vec<char>>` |
| `fromChar` | treatment | `chars Stream<Vec<char>>` | `text Stream<string>` |
| `toUtf8` | treatment | `text Stream<string>` | `encoded Stream<byte>` |
| `fromUtf8` | treatment | `encoded Stream<byte>` | `text Stream<string>` *(lossy)* |
| `\|toChar(text: string) -> Vec<char>` | function | — | — |
| `\|fromChar(chars: Vec<char>) -> string` | function | — | — |
| `\|toUtf8(text: string) -> Vec<byte>` | function | — | — |
| `\|fromUtf8(encoded: Vec<byte>) -> string` | function | — | *(lossy)* |

---

### `std/text/convert/char`

```mel
use std/text/convert/char::toString
```

| Element | Kind | Input | Output |
|---------|------|-------|--------|
| `toString` | treatment | `chars Stream<char>` | `text Stream<string>` |
| `fromString` | treatment | `text Stream<string>` | `chars Stream<char>` |
| `toUtf8` | treatment | `chars Stream<char>` | `encoded Stream<byte>` |
| `\|toUtf8(char: char) -> Vec<byte>` | function | — | — |
| `\|fromUtf8(encoded: Vec<byte>) -> Vec<char>` | function | — | — |

---

### `std/text/compare`

```mel
use std/text/compare::contains
use std/text/compare::exact
```

| Element | Kind | Inputs | Outputs |
|---------|------|--------|---------|
| `contains(substring: string)` | treatment | `text Stream<string>` | `contains Stream<bool>` |
| `exact(pattern: string)` | treatment | `text Stream<string>` | `matches Stream<bool>` |
| `startsWith(pattern: string)` | treatment | `text Stream<string>` | `matches Stream<bool>` |
| `endsWith(pattern: string)` | treatment | `text Stream<string>` | `matches Stream<bool>` |
| `\|contains(text: string, substring: string) -> bool` | function | — | — |
| `\|exact(text: string, pattern: string) -> bool` | function | — | — |
| `\|startsWith`, `\|endsWith` | function | — | — |

---

### `std/text/compare/char`

| Element | Kind | Input | Output |
|---------|------|-------|--------|
| `exact(reference: char)` | treatment | `char Stream<char>` | `matches Stream<bool>` |
| `isAlphabetic` | treatment | `char Stream<char>` | `result Stream<bool>` |
| `isAlphanumeric` | treatment | `char Stream<char>` | `result Stream<bool>` |
| `isAscii` | treatment | `char Stream<char>` | `result Stream<bool>` |
| `isControl` | treatment | `char Stream<char>` | `result Stream<bool>` |
| `isDigit(base: u32)` | treatment | `char Stream<char>` | `result Stream<bool>` |
| `isLowercase` | treatment | `char Stream<char>` | `result Stream<bool>` |
| `isUppercase` | treatment | `char Stream<char>` | `result Stream<bool>` |
| `isWhitespace` | treatment | `char Stream<char>` | `result Stream<bool>` |

All also available as functions: `\|isAlphabetic(char) -> bool`, etc.

---

### `std/types`

```mel
use std/types::|min
use std/types::|max
```

| Function | Description |
|----------|-------------|
| `\|min<B: Bounded>() -> B` | Minimum value for type `B` |
| `\|max<B: Bounded>() -> B` | Maximum value for type `B` |

### `std/types/float`

| Function | Description |
|----------|-------------|
| `\|infinity<F: Float>() -> F` | Positive infinity |
| `\|negInfinity<F: Float>() -> F` | Negative infinity |
| `\|nan<F: Float>() -> F` | Not-a-number |

### `std/types/char`

| Function | Description |
|----------|-------------|
| `\|replacementCharacter() -> char` | Unicode replacement character (`U+FFFD`) |

---

## `fs` — Filesystem

### `fs/local`

```mel
use fs/local::|local_filesystem
```

| Function | Signature |
|----------|-----------|
| `\|localFilesystem(path: Option<string>) -> Option<FileSystem>` | Returns a local filesystem handle rooted at `path`, or CWD if `None`. |

---

### `fs/filesystem`

```mel
use fs/filesystem::FileSystem
```

**`FileSystem` data type** — opaque handle to a filesystem (local or remote). Required by `read`, `write`, `scan`, `create`.

---

### `fs/file`

```mel
use fs/file::read
use fs/file::write
```

**`read`:**

| Inputs | Outputs |
|--------|---------|
| `path Block<string>`, `filesystem Block<FileSystem>` | `data Stream<byte>`, `reached Block<void>`, `completed Block<void>`, `failed Block<void>`, `finished Block<void>`, `errors Stream<string>` |

- `reached`: emitted when file is found and open.
- `completed`: emitted after all bytes sent successfully.
- `failed`: emitted on error (path not found, permission, etc.).
- `finished`: emitted in both cases (after `completed` or `failed`).

**`write(append: bool, create: bool, new: bool)`:**

| Inputs | Outputs |
|--------|---------|
| `path Block<string>`, `filesystem Block<FileSystem>`, `data Stream<byte>` | `completed Block<void>`, `failed Block<void>`, `finished Block<void>`, `errors Stream<string>`, `amount Stream<u128>` |

- `amount`: running total of bytes written.

---

### `fs/path`

```mel
use fs/path::composition
use fs/path::|extension
```

**`composition` treatment:**

| Input | Outputs |
|-------|---------|
| `path Stream<string>` | `extension Stream<Option<string>>`, `fileName Stream<Option<string>>`, `fileStem Stream<Option<string>>`, `parent Stream<Option<string>>` |

**Functions:**

| Function | Returns |
|----------|---------|
| `\|extension(path: string) -> Option<string>` | |
| `\|fileName(path: string) -> Option<string>` | |
| `\|fileStem(path: string) -> Option<string>` | |
| `\|parent(path: string) -> Option<string>` | |

**Other treatments:**

| Treatment | Input | Output |
|-----------|-------|--------|
| `exists` | `path Stream<string>` | `exists Stream<bool>` |
| `meta` | `path Stream<string>` | `isDir Stream<bool>`, `isFile Stream<bool>`, `length Stream<u64>` |

---

### `fs/dir`

```mel
use fs/dir::scan
use fs/dir::create
```

**`scan(recursive: bool, followLinks: bool)`:**

| Inputs | Outputs |
|--------|---------|
| `path Block<string>`, `filesystem Block<FileSystem>` | `entries Stream<string>`, `completed Block<void>`, `failed Block<void>`, `finished Block<void>`, `errors Stream<string>` |

**`create(recursive: bool)`:**

| Inputs | Outputs |
|--------|---------|
| `path Block<string>`, `filesystem Block<FileSystem>` | `success Block<void>`, `failure Block<void>`, `error Block<string>` |

---

## `http` — HTTP Client and Server

### `http/server`

```mel
use http/server::HttpServer
use http/server::start
use http/server::connection
use http/server::@HttpRequest
```

**`@HttpRequest` context** — available in all tracks initiated by an HTTP server source.

| Field | Type | Description |
|-------|------|-------------|
| `@HttpRequest[id]` | `u128` | Unique connection identifier |
| `@HttpRequest[route]` | `string` | Matched route pattern |
| `@HttpRequest[path]` | `string` | Actual request path |
| `@HttpRequest[parameters]` | `StringMap` | Path/query parameters extracted from route |
| `@HttpRequest[method]` | `HttpMethod` | HTTP method |

**`HttpServer` model** — parameters:

| Parameter | Type | Default |
|-----------|------|---------|
| `host` | `Ip` | *(required)* |
| `port` | `u16` | *(required)* |

Sources:
- `incoming(method: HttpMethod, route: string)` — creates a new track per matching request, provides `@HttpRequest`.
- `failedBinding` — fires `failed Block<void>`, `error Block<string>` if server can't bind.

**Treatments:**

| Treatment | Inputs | Outputs | Notes |
|-----------|--------|---------|-------|
| `start[http_server: HttpServer]` | `trigger Block<void>` | *(none)* | Starts listening |
| `connection[http_server: HttpServer](method: HttpMethod, route: string)` | `status Block<HttpStatus>`, `headers Block<StringMap>`, `data Stream<byte>` | `data Stream<byte>` | High-level: handles one route bidirectionally |
| `outgoing(id: u128)[http_server: HttpServer]` | `status Block<HttpStatus>`, `headers Block<StringMap>`, `data Stream<byte>` | *(none)* | Low-level response writer |

The `connection` treatment is the standard way to handle HTTP routes. It wraps `incoming` + `outgoing`.

---

### `http/client`

```mel
use http/client::HttpClient
use http/client::request
use http/client::requestWithBody
```

**`HttpClient` model** — parameters:

| Parameter | Type | Default |
|-----------|------|---------|
| `baseUrl` | `Option<string>` | `None` |
| `tcpNoDelay` | `bool` | `false` |
| `headers` | `StringMap` | *(empty)* |

**`request(method: HttpMethod)[client: HttpClient]`:**

| Inputs | Outputs |
|--------|---------|
| `url Block<string>`, `reqHeaders Block<StringMap>` | `resHeaders Block<StringMap>`, `data Stream<byte>`, `completed Block<void>`, `failed Block<void>`, `finished Block<void>`, `error Block<string>`, `status Block<HttpStatus>` |

**`requestWithBody(method: HttpMethod)[client: HttpClient]`:**

Same as `request` plus `body Stream<byte>` input.

---

### `http/method`

```mel
use http/method::HttpMethod
use http/method::|get
```

**`HttpMethod` data type.**

| Function | Returns |
|----------|---------|
| `\|method(name: string) -> Option<HttpMethod>` | Parse from string |
| `\|delete() -> HttpMethod` | |
| `\|get() -> HttpMethod` | |
| `\|head() -> HttpMethod` | |
| `\|options() -> HttpMethod` | |
| `\|patch() -> HttpMethod` | |
| `\|post() -> HttpMethod` | |
| `\|put() -> HttpMethod` | |
| `\|trace() -> HttpMethod` | |

---

### `http/status`

```mel
use http/status::HttpStatus
use http/status::|ok
```

**`HttpStatus` data type.**

| Function | Code |
|----------|------|
| `\|status(code: u16) -> Option<HttpStatus>` | Parse from code |
| `\|ok() -> HttpStatus` | 200 |
| `\|movedPermanently() -> HttpStatus` | 301 |
| `\|temporaryRedirect() -> HttpStatus` | 307 |
| `\|permanentRedirect() -> HttpStatus` | 308 |
| `\|forbidden() -> HttpStatus` | 403 |
| `\|notFound() -> HttpStatus` | 404 |

---

## `net` — Network Primitives

### `net/ip`

```mel
use net/ip::Ip
use net/ip::|localhost_ipv4
use net/ip::|from_ipv4
```

**Data types:** `Ip`, `Ipv4`, `Ipv6`.

**Functions:**

| Function | Signature |
|----------|-----------|
| `\|fromIpv4(addr: Ipv4) -> Ip` | Wrap IPv4 into `Ip` |
| `\|fromIpv6(addr: Ipv6) -> Ip` | Wrap IPv6 into `Ip` |
| `\|asIpv4(addr: Ip) -> Option<Ipv4>` | Extract IPv4 if present |
| `\|asIpv6(addr: Ip) -> Option<Ipv6>` | Extract IPv6 if present |
| `\|isIpv4(addr: Ip) -> bool` | |
| `\|isIpv6(addr: Ip) -> bool` | |
| `\|ipv4(a: u8, b: u8, c: u8, d: u8) -> Ipv4` | Construct from octets |
| `\|ipv6(a…h: u16) -> Ipv6` | Construct from 8 groups |
| `\|toIpv4(text: string) -> Option<Ipv4>` | Parse string |
| `\|toIpv6(text: string) -> Option<Ipv6>` | Parse string |
| `\|localhostIpv4() -> Ipv4` | `127.0.0.1` |
| `\|unspecifiedIpv4() -> Ipv4` | `0.0.0.0` |
| `\|localhostIpv6() -> Ipv6` | `::1` |
| `\|unspecifiedIpv6() -> Ipv6` | `::` |

Streaming treatments available for all conversion and predicate functions.

---

## `process` — Subprocess Execution

### `process/local`

```mel
use process/local::|localExecutor
```

| Function | Signature |
|----------|-----------|
| `\|localExecutor() -> Option<Executor>` | Returns an executor for the local OS. |

### `process/exec`

```mel
use process/exec::Executor
use process/exec::execOneTerminable
use process/exec::spawnOneTerminable
```

**`Executor` data type** — opaque handle to an execution environment.

Key treatments (abbreviated — all require a `Block<Executor>` input named `executor` and a `command` input):

| Treatment | Extra Inputs | Outputs |
|-----------|-------------|---------|
| `execOneTerminable` | `executor Block<Executor>`, `command Block<Command>`, `environment Block<Option<Environment>>`, `terminate Block<void>` | `started`, `finished`, `completed`, `failed`, `terminated Block<void>`, `error Block<string>`, `exit Block<Option<i32>>` |
| `execTerminable` | same but `command Stream<Command>` | `exit Stream<Option<i32>>`, others as above |
| `spawnOneTerminable` | adds `stdin Stream<byte>` | adds `stdout Stream<byte>`, `stderr Stream<byte>` |
| `spawnTerminable` | streaming commands version | same |

---

## `regex` — Regular Expressions

```mel
use regex::matches
use regex::find
use regex::capture
use regex::replace
```

| Element | Kind | Inputs | Outputs |
|---------|------|--------|---------|
| `matches(regex: string)` | treatment | `text Stream<string>` | `matches Stream<bool>`, `error Block<string>` |
| `find(regex: string)` | treatment | `text Stream<string>` | `found Stream<Option<string>>`, `error Block<string>` |
| `capture(regex: string)` | treatment | `text Stream<string>` | `captured Stream<Option<StringMap>>`, `error Block<string>` |
| `replace(regex: string, replacer: string)` | treatment | `text Stream<string>` | `replaced Stream<string>`, `error Block<string>` |
| `\|matches(text: string, regex: string) -> bool` | function | — | — |
| `\|find(text: string, regex: string) -> Option<string>` | function | — | — |
| `\|capture(text: string, regex: string) -> Option<StringMap>` | function | — | — |
| `\|replace(text: string, regex: string, replacer: string) -> Option<string>` | function | — | — |

---

## `json` — JSON

```mel
use json::Json
use json::toJson
use json::validate
use json/value::|null
```

**`Json` data type** — implements `ToString`, `TryToString`, `TryToBool`, `TryToI64`, `TryToU64`, `TryToF64`, `Display`, `Deserialize`.

| Element | Kind | Inputs | Outputs |
|---------|------|--------|---------|
| `toJson` | treatment | `text Stream<string>` | `json Stream<Option<Json>>`, `error Stream<Option<string>>` |
| `validate` | treatment | `text Stream<string>` | `isJson Stream<bool>` |
| `\|toJson(text: string) -> Option<Json>` | function | — | — |

**`json/value` functions:**

| Function | Returns |
|----------|---------|
| `\|null() -> Json` | JSON `null` |
| `\|bool(value: bool) -> Json` | JSON boolean |
| `\|integer(value: i64) -> Json` | JSON integer |
| `\|float(value: f64) -> Json` | JSON float |
| `\|string(value: string) -> Json` | JSON string |
| `\|array(values: Vec<Json>) -> Json` | JSON array |
| `\|object(entries: Map) -> Json` | JSON object |

---

## `encoding` — Text Encoding

```mel
use encoding::decode
use encoding::encode
```

| Treatment | Parameters | Inputs | Outputs |
|-----------|-----------|--------|---------|
| `decode(encoding: string = "utf-8")` | — | `data Stream<byte>` | `text Stream<string>` |
| `encode(encoding: string = "utf-8", replace: bool = false)` | — | `text Stream<string>` | `data Stream<byte>` |

Supported encodings follow the WHATWG Encoding Standard labels (e.g. `"utf-8"`, `"latin1"`, `"utf-16be"`, etc.).

---

## `sql` — SQL Database

```mel
use sql::SqlPool
use sql::fetch
use sql::execute
```

**`SqlPool` model** — parameters:

| Parameter | Type | Default |
|-----------|------|---------|
| `url` | `string` | *(required)* |
| `maxConnections` | `u32` | `10` |
| `minConnections` | `u32` | `0` |
| `acquireTimeout` | `u64` | `10000` *(ms)* |
| `idleTimeout` | `Option<u64>` | `600000` *(ms)* |
| `maxLifetime` | `Option<u64>` | `1800000` *(ms)* |

Sources: `connected`, `failure`, `closed`.

**Connection control:**

| Treatment | Inputs | Notes |
|-----------|--------|-------|
| `connect[sql_pool: SqlPool]` | `trigger Block<void>` | Opens the pool |
| `close[sql_pool: SqlPool]` | `trigger Block<void>` | Closes the pool |

**Query treatments:**

All query treatments share common outputs: `finished Block<void>`, `completed Block<void>`, `failed Block<void>`, `error Block<string>` (or `errors Stream<string>` for multi-row variants).

| Treatment | Extra params | Inputs | Extra outputs |
|-----------|-------------|--------|---------------|
| `executeRaw(sql: string)[sql_pool]` | — | `trigger Block<void>` | `affected Block<u64>` |
| `execute(sql, bindings: Vec<string>, bindSymbol: string = "?")[sql_pool]` | — | `bind Block<Map>` | `affected Block<u64>` |
| `executeEach(sql, bindings, bindSymbol, stopOnFailure: bool)[sql_pool]` | — | `bind Stream<Map>` | `affected Stream<u64>`, `errors Stream<string>` |
| `executeBatch(base, batch, bindings, bindSymbol, bindLimit, separator, stopOnFailure)[sql_pool]` | — | `bind Stream<Map>` | `affected Stream<u64>` |
| `fetch(sql, bindings, bindSymbol)[sql_pool]` | — | `bind Block<Map>` | `data Stream<Map>`, `errors Stream<string>` |
| `fetchBatch(base, batch, bindings, bindLimit, bindSymbol, separator, stopOnFailure)[sql_pool]` | — | `bind Stream<Map>` | `data Stream<Map>` |

The `bindings` parameter lists the Map keys to bind, in order; `bindSymbol` is the placeholder in the SQL string (default `?`, use `$1`-style for PostgreSQL by passing ordered keys).

---

## `javascript` — JavaScript Engine

```mel
use javascript::JavaScriptEngine
use javascript::process
```

**`JavaScriptEngine` model** — parameters:

| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | `string` | JavaScript source code defining functions |

**`process[engine: JavaScriptEngine](code: string)`:**

| Inputs | Outputs |
|--------|---------|
| `value Stream<Json>` | `result Stream<Option<Json>>` |

Evaluates `code` for each `value` (the value is bound as `value` in the JS scope). Returns `None` if evaluation fails or result is `undefined`.

---

## `distrib` — Distributed Computing

```mel
use distrib::DistributionEngine
use distrib::start
use distrib::distribute
use distrib::recvStream
use distrib::sendStream
```

**`DistributionEngine` model** — parameters:

| Parameter | Type | Description |
|-----------|------|-------------|
| `treatment` | `string` | Fully-qualified treatment name to distribute |
| `version` | `string` | Version constraint |

**Treatments:**

| Treatment | Inputs | Outputs |
|-----------|--------|---------|
| `start(params: Map)[distributor: DistributionEngine]` | `access Block<Access>` | `ready Block<void>`, `failed Block<void>`, `error Block<string>` |
| `stop[distributor: DistributionEngine]` | `trigger Block<void>` | *(none)* |
| `distribute[distributor: DistributionEngine]` | `trigger Block<void>` | `distributionId Block<u64>`, `failed Block<void>`, `error Block<string>` |
| `recvStream<D: Deserialize>(name: string)[distributor]` | `distributionId Block<u64>` | `data Stream<D>` |
| `recvBlock<D: Deserialize>(name: string)[distributor]` | `distributionId Block<u64>` | `data Block<D>` |
| `sendStream<S: Serialize>(name: string)[distributor]` | `distributionId Block<u64>`, `data Stream<S>` | *(none)* |
| `sendBlock<S: Serialize>(name: string)[distributor]` | `distributionId Block<u64>`, `data Block<S>` | *(none)* |

---

## `work` — Work Queues and Cloud Platform

The `work` package provides access to the Mélodium cloud platform APIs, container execution, and distributed work coordination. It is used for running treatments in containers, managing cloud resources, and accessing the Mélodium API.

Key sub-modules: `access`, `api`, `compose`, `container`, `distant`, `reporting`, `resources`.

Refer to https://doc.melodium.tech/latest/en/ for the current work package API surface, as it evolves with the platform.

---

## Notes on Mel naming conventions

Rust uses `snake_case`; Mélodium uses `camelCase` for multi-word identifiers. When importing from Rust-implemented packages, always use the Mélodium-cased names:

| Rust | Mélodium |
|------|----------|
| `local_filesystem` | `localFilesystem` |
| `log_info_message` | `logInfoMessage` |
| `to_utf8` | `toUtf8` |
| `is_positive` | `isPositive` |
| `request_with_body` | `requestWithBody` |

When in doubt, check https://doc.melodium.tech/latest/en/ for the exact Mélodium-facing name.
