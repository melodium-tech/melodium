# 02: Flow & Generics

**Concepts introduced:** generic treatments (`<N: Trait>`), `std/flow` combinators (`filter`, `merge`, `fill`), `std/ops` arithmetic & comparison, writing your own reusable treatments.

Generates the integers `1..upper`, splits them around `threshold`, nudges each half by a different offset, and merges the two halves back into a single stream: using two small generic treatments defined right in this file and reused twice each.

## What it does

```
melodium run Compo.toml --upper 10 --threshold 5 --offset_above 100 --offset_below=-100
```

A negative value needs the `--flag=value` form; `--offset_below -100` fails to parse (the CLI reads `-100` as a short-flag cluster, not a value).

- Builds the sequence `1, 2, …, upper` (same `generate` + `count` trick as example 01).
- `aboveThreshold<N: PartialOrder>` splits it into two streams around `threshold`.
- `shift<N: Add>` adds a different offset to each half.
- `merge` recombines both halves into one stream, in no particular order.

## How it is built

No models here either: everything is stateless treatments, functions, and two custom generic treatments.

### Data flow

```
1..upper ──▶ aboveThreshold<i64> ──▶ above ──▶ shift(+100) ──┐
                                │                              ├─▶ merge ──▶ log
                                └──▶ below ──▶ shift(-100) ──┘
```

## Runtime behaviour

1. `generate` produces `upper` placeholder values, and `count` numbers them starting at 1, exactly as example 01 documents. Verified with `melodium run --upper 10 --threshold 5`: `at-or-below-threshold` logs `1, 2, 3, 4, 5` and `above-threshold` logs `6, 7, 8, 9, 10`, the symmetric split `1..10` around `5` implies.
2. `aboveThreshold<i64>` is instantiated once (`split`) but is written generically: it never mentions `i64` in its body. Inside, it builds a same-length stream of the `threshold` value with `toVoid` + `fill` (so it can compare element-by-element with `greaterThan`), then uses that boolean stream to `filter` the input into `above`/`below`.
3. `shift<i64>` is instantiated twice (`bumpAbove`, `bumpBelow`) with two different `offset` values: same treatment, same trick with `fill`, this time feeding `std/ops/num::add`.
4. `merge` interleaves the two shifted streams unpredictably; run the program twice and the log order may differ, which is expected: Mélodium streams have no implicit ordering guarantee across branches.
5. A final `count` numbers the merged stream just to show it is a stream like any other.

### Key Mélodium patterns used

- **Generic treatments with trait bounds**: `aboveThreshold<N: PartialOrder>` and `shift<N: Add>` are written once and instantiated at different types (or, here, the same type twice with different parameters) without duplicating any logic.
- **`toVoid` + `fill`**: the standard way to turn a constant into a stream whose length matches another stream, so element-wise treatments like `greaterThan`/`add` (which read one value at a time from *two* streams) have a value to pair with each item.
- **`Self` fan-out**: inside `aboveThreshold`, `Self.value` is read three times (into `asVoid`, `isAbove`, and `partition`); reading the same input more than once is allowed, only *writing* to the same input twice is not.
- **`filter`**: splits one stream into `accepted`/`rejected` according to a parallel `bool` stream.
- **`merge`**: recombines two streams without a defined interleaving order.

Next: [03_text_and_files](../03_text_and_files/) introduces text processing, regular expressions, and file I/O.
