# Traits

Traits are intrinsic abilities of a given type. They define what operations a type supports: whether it can be added, compared, converted, serialized, displayed, and so on.

Any type can implement any trait, as long as it has a logical meaning for that type. For example, `u32` implements `Add` (addition makes sense for integers), but `bool` does not implement `Div` (dividing booleans has no meaning).

## Why traits matter

Traits are most useful in two situations:

**Constraining generics.** When writing a generic treatment that works on any numeric type, traits let you express that constraint precisely. A generic `<N: Add + PartialOrder>` accepts any type that supports addition and comparison, and nothing else.

```mel
// Only types that implement SaturatingToI64 and Float can be used here.
treatment demonstration<N: Float + SaturatingToI64>()
  input  value:   Stream<N>
  output integer: Stream<i64>
{
    saturatingToI64<N>()
    Self.value -> saturatingToI64.value,into -> Self.integer
}
```

**Understanding what a type can do.** When using a type from a package, its documentation lists its implemented traits. If a type implements `Serialize`, you can pass it to treatments that serialize data. If it implements `Equality`, you can compare two instances.

## Trait families

Traits are organized in families by purpose:

- **Conversions** (`ToI8`, `ToF64`, `ToString`, ...): infallible type conversions.
- **Fallible conversions** (`TryToI8`, `TryToU32`, ...): conversions that may fail, returning `Option<T>`.
- **Arithmetic** (`Add`, `Sub`, `Mul`, `Div`, `Neg`, `Pow`, ...): mathematical operations.
- **Checked arithmetic** (`CheckedAdd`, `CheckedSub`, ...): arithmetic that returns `Option<T>` on overflow.
- **Saturating arithmetic** (`SaturatingAdd`, `SaturatingMul`, ...): arithmetic that clamps at the type boundary.
- **Comparison** (`PartialEquality`, `Equality`, `PartialOrder`, `Order`): equality and ordering.
- **Numeric** (`Signed`, `Float`, `Bounded`): numeric classification traits.
- **Serialization** (`Serialize`, `Deserialize`): data encoding and decoding.
- **Display** (`ToString`): human-readable representation.

The complete list of traits and their implementation per core type is available in the [Trait list](full_list.md). In practice, the documentation page for each type is the most direct way to see which traits it implements.
