# Core types

Mélodium have multiple core data types, shared across four main categories:
- unsigned integers,
- signed integers,
- floating-point numbers,
- textual data,

on which `bool`, `byte` and `void` can be added.

All those types are described across their respective section, each type meets a specific purpose.

| Unsigned integers | Signed integers | Floating-point numbers | Text   | Logic |
|-------------------|-----------------|------------------------|--------|-------|
| `u8`                | `i8`              | `f32`                    | `char`   | `byte`  |
| `u16`               | `i16`             | `f64`                    | `string` | `bool`  |
| `u32`               | `i32`             |                        |        | `void`  |
| `u64`               | `i64`             |                        |        |       |
| `u128`              | `i128`            |                        |        |       |

## Byte

| Type   | Values | Size |
|--------|-------|---|
| `byte`   | Any 8-bits data | 8 bits / 1 byte |

A `byte` is basically the most atomic unit of data manipulable through Mélodium.
It represents any 8-bits data, without more assumption on what it could be.

## Bool

| Type   | Values | Size |
|--------|--------|------|
| `bool`   | `true` or `false` | 8 bits / 1 byte |

A `bool` is a boolean value that can be either set to `true` or `false`.
Conversion treatments are available for `bool`s to be turned into bytes, numbers, or any kind of value.

## Void

| Type   | Values | Size |
|--------|--------|------|
| `void`   | _None_ | 0 bit / 0 byte |

`void` does not hold any value. It just signals that _something_ is happening or has happened. It is used in connections to transmit triggers or continuation indicators.

`Block<void>` carries a single trigger: it fires once to signal that an event occurred, such as initialization completing or a condition being met. `Stream<void>` carries a continuous series of signals, for example to indicate that items are being processed without conveying the items themselves.

```mel
use std/flow::trigger
use std/flow::emit

// trigger produces a Stream<void> that fires once per item passing through.
// emit<void> can fire a Block<void> to signal that setup is done.
trigger<string>()
emitDone: emit<void>(value=_)
```

_Reference for [trigger](https://doc.melodium.tech/latest/en/std/flow/trigger.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html)_

## Unsigned integers

| Type   | Range | Size |
|--------|-------|---|
| `u8`   | 0 to 2⁸-1 (255) | 8 bits / 1 byte |
| `u16`  | 0 to 2¹⁶-1 (65,535) | 16 bits / 2 bytes |
| `u32`  | 0 to 2³²-1 (4,294,967,295) | 32 bits / 4 bytes |
| `u64`  | 0 to 2⁶⁴-1 ( > 18×10¹⁸) | 64 bits / 8 bytes |
| `u128` | 0 to 2¹²⁸-1 ( > 34×10³⁷) | 128 bits / 16 bytes |

## Signed integers

| Type   | Range | Size |
|--------|-------|---|
| `i8`   | -2⁷ (-128) to 2⁷-1 (127) | 8 bits / 1 byte |
| `i16`  | -2¹⁵ (-32,768) to 2¹⁵-1 (32,767) | 16 bits / 2 bytes |
| `i32`  | -2³¹ (-2,147,483,648) to 2³¹-1 (2,147,483,647) | 32 bits / 4 bytes |
| `i64`  | -2⁶³ ( ≈ -9×10¹⁵) to 2⁶³-1 ( ≈ 9×10¹⁵) | 64 bits / 8 bytes |
| `i128` | -2¹²⁷ ( ≈ -34×10³⁷) to 2¹²⁷-1 ( ≈ 34×10³⁷) | 128 bits / 16 bytes |

## Floating-point numbers

| Type   | Values | Size |
|--------|-------|---|
| `f32`  | _See description_ | 32 bits / 4 bytes |
| `f64`  | _See description_ | 64 bits / 8 bytes |

Floating-point numbers are defined in IEEE 754-2008. They can mostly be considered as _decimal_ numbers, for a deeper explanation, please refers to the [Single-precision floating-point format](https://en.wikipedia.org/wiki/Single-precision_floating-point_format) (for `f32`) and [Double-precision floating-point format](https://en.wikipedia.org/wiki/Double-precision_floating-point_format) (for `f64`) articles on Wikipedia.

They can store positive or negative values, but also be in one of those three states:
- _positive infinity_, can be result of something like `1.0/0.0`;
- _negative infinity_, can be result of something like `-1.0/0.0`;
- _not a number_, can be result of a square root of negative number (aka. complex number).

## Textual data

| Type   | Values | Size |
|--------|-------|---|
| `char`  | Any valid Unicode scalar value | 32 bits / 4 bytes |
| `string`  | Any valid UTF-8 text | _Variable_ |

All textual information is represented as Unicode. A `char` uses 4 bytes to store any Unicode scalar value, as defined in [Unicode Standard](https://www.unicode.org/glossary/#unicode_scalar_value). Unlike many other programming languages, Mélodium does not assume a char and a byte (nor combination of bytes) to be equivalent at all, for many reasons such as:
- a byte only have 256 values, while all human languages combined have much more "letters";
- a letter in Unicode Text Format can be up to 4 bytes;
- many values are illegal according to Unicode;
- Unicode standard provide a strong universality of what textual data can be represented;
- making data types reliable, each one having its own purpose, then `char` guarantees valid text data while `byte` only assume it is data.

The `string` data type can represent any UTF-8 text and its size depends on the length of the text. Interestingly, `string`s are not a combination of `char`s, but real UTF-8 strings. Taking the text `Mélodium` and putting it as vector of chars, 32 bytes (8 chars × 4 bytes) are used, but as string only 9 bytes. This technical subtility is transparent for users and conversion treatments are provided if needed.

Mélodium can handle many encodings through its encoders and decoders, taking and providing byte streams.

[//]: # (Comparison with other languages)
