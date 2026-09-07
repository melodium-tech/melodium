# Traits list

Here is the exhaustive list of Mélodium traits.

Some traits are grouped by family, as they behave the same way.
The behavior is then explained with general meaning, and each individual trait description exposes some specificities.

## Infaillible conversions

_Infaillible conversions_ designates conversions that are guaranteed to succeed, for which for every _X_ initial state there exist a _Y_ result state.

> While these conversions are guaranteed to succeed, they are not guaranteed to be reversible.
> As example, `u8` implements `ToI64`, but `i64` does not implements `ToU8`.

These traits are mostly useful trough treatments and functions of the [`conv` area](https://doc.melodium.tech/latest/en/std/conv/index.html).

### `ToI8`

Type is able to convert itself into a `i8` value.

| Implemented by |
|----------------|
| `i8` |

### `ToI16`

Type is able to convert itself into a `i16` value.

| Implemented by |
|----------------|
| `i8` |
| `u8` |

### `ToI32`

Type is able to convert itself into a `i32` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `u8` |
| `u16` |

### `ToI64`

Type is able to convert itself into a `i64` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `u8` |
| `u16` |
| `u32` |

### `ToI128`

Type is able to convert itself into a `i128` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |

### `ToU8`

Type is able to convert itself into a `u8` value.

| Implemented by |
|----------------|
| `u8` |
| `byte` |

### `ToU16`

Type is able to convert itself into a `u16` value.

| Implemented by |
|----------------|
| `u8` |
| `u16` |

### `ToU32`

Type is able to convert itself into a `u32` value.

| Implemented by |
|----------------|
| `u8` |
| `u16` |
| `u32` |

### `ToU64`

Type is able to convert itself into a `u64` value.

| Implemented by |
|----------------|
| `u8` |
| `u16` |
| `u32` |
| `u64` |

### `ToU128`

Type is able to convert itself into a `u128` value.

| Implemented by |
|----------------|
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `ToF32`

Type is able to convert itself into a `f32` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `ToF64`

Type is able to convert itself into a `f64` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `ToBool`

Type is able to convert itself into a `bool` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `bool` |
| `byte` |

### `ToByte`

Type is able to convert itself into a `byte` value.

| Implemented by |
|----------------|
| `u8` |
| `bool` |
| `byte` |

### `ToChar`

Type is able to convert itself into a `char` value.

| Implemented by |
|----------------|
| `char` |

### `ToString`

Type is able to convert itself into a `string` value.

> While it is useful to have a conversion to `string` for a type, it should not be considered 
> as a “readable” version of the value, and so not be confused with `Display` trait, that is
> especially designed for this purpose.

| Implemented by |
|----------------|
| `void` |
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |
| `bool` |
| `byte` |
| `char` |
| `string` |

## Faillible conversions

_Faillible conversions_ designates conversions that are possible without being guaranteed to succeed.

Those conversions are useful to get an `Option<T>` result, where `T` is the target type, containing a value if conversion succeed, or _none_ if it cannot be done.

These traits are mostly useful trough treatments and functions of the [`conv` area](https://doc.melodium.tech/latest/en/std/conv/index.html).

> A general rule is that when a type implements a `To*` trait, it also does for its `TryTo*` counterpart.

### `TryToI8`

Type can try to convert itself into a `i8` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToI16`

Type can try to convert itself into a `i16` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToI32`

Type can try to convert itself into a `i32` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToI64`

Type can try to convert itself into a `i64` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToI128`

Type can try to convert itself into a `i128` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToU8`

Type can try to convert itself into a `u8` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToU16`

Type can try to convert itself into a `u16` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToU32`

Type can try to convert itself into a `u32` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToU64`

Type can try to convert itself into a `u64` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToU128`

Type can try to convert itself into a `u128` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToF32`

Type can try to convert itself into a `f32` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToF64`

Type can try to convert itself into a `f64` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `TryToBool`

Type can try to convert itself into a `bool` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `bool` |
| `byte` |

### `TryToByte`

Type can try to convert itself into a `byte` value.

| Implemented by |
|----------------|
| `u8` |
| `bool` |
| `byte` |

### `TryToChar`

Type can try to convert itself into a `char` value.

| Implemented by |
|----------------|
| `char` |

### `TryToString`

Type can try to convert itself into a `string` value.

| Implemented by |
|----------------|
| `void` |
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |
| `bool` |
| `byte` |
| `char` |
| `string` |

## Saturating conversions

_Saturating conversions_ are specific kind of conversions where a type can try to convert itself into another one, and instead of failing the conversion if its value cannot be rendered into the target type, push to the closest bound of that type according to initial value.

This kind of trait exists to target numeric types.

### `SaturatingToI8`

Type can make a saturating conversion to `i8` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToI16`

Type can make a saturating conversion to `i16` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToI32`

Type can make a saturating conversion to `i32` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToI64`

Type can make a saturating conversion to `i64` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToI128`

Type can make a saturating conversion to `i128` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToU8`

Type can make a saturating conversion to `u8` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToU16`

Type can make a saturating conversion to `u16` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToU32`

Type can make a saturating conversion to `u32` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToU64`

Type can make a saturating conversion to `u64` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToU128`

Type can make a saturating conversion to `u128` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToF32`

Type can make a saturating conversion to `f32` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `SaturatingToF64`

Type can make a saturating conversion to `f64` value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

## `Bounded`

The type have minimum and maximum limits.

This trait is mostly useful trough functions of the [`types` area](https://doc.melodium.tech/latest/en/std/types/index.html).

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |
| `byte` |

## `Binary`

Type can be used for meaningful mathematical binary operations.

This trait is mostly useful trough treatments and functions of the [`bin` area](https://doc.melodium.tech/latest/en/std/ops/bin/index.html).

| Implemented by |
|----------------|
| `bool` |
| `byte` |

## `Signed`

Type is signed, meaning it can have _negative values_.

This trait is mostly useful trough treatments and functions of the [`num` area](https://doc.melodium.tech/latest/en/std/ops/num/index.html).

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `f32` |
| `f64` |

## `Float`

Type represent _floating-point_ values, and is able to proceed to _floating-point_ arithmetic.

This trait is mostly useful trough treatments and functions of the [`float` area](https://doc.melodium.tech/latest/en/std/ops/float/index.html).

| Implemented by |
|----------------|
| `f32` |
| `f64` |

## `PartialEquality`

Type can be compared to itself and tell wether both values are equal or not.

`PartialEquality` does not require a full equivalence between all the value of a type.
As example, for `f32` and `f64`, `NaN` is not equal to itself. As such, those types implements `PartialEquality` but _not_ `Equality`.

This trait is mostly useful trough treatments and functions of the [`ops` area](https://doc.melodium.tech/latest/en/std/ops/index.html).

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |
| `bool` |
| `byte` |
| `char` |
| `string` |

## `Equality`

Type can be compared to itself and tell wether both values are equal or not.

`Equality` require a full equivalence between all the value of a type, meaning any _X_ value is always equal to itself and always different from any other.

This trait is mostly useful trough treatments and functions of the [`ops` area](https://doc.melodium.tech/latest/en/std/ops/index.html).

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `bool` |
| `byte` |
| `char` |
| `string` |

## `PartialOrder`

Type implements [partial order](https://en.wikipedia.org/wiki/Partially_ordered_set#Partial_order), meaning values of this type can be compared in a way telling if one is greater or lesser to another, strictly or not.

This trait is mostly useful trough treatments and functions of the [`ops` area](https://doc.melodium.tech/latest/en/std/ops/index.html).

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |
| `bool` |
| `byte` |
| `char` |
| `string` |

## `Order`

Type implements [total order](https://en.wikipedia.org/wiki/Total_order), meaning values of this type can be compared, and absolute minimums and maximums in a set of those values can be found.

This trait is mostly useful trough treatments and functions of the [`ops` area](https://doc.melodium.tech/latest/en/std/ops/index.html).

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |
| `bool` |
| `byte` |
| `char` |
| `string` |

## Basic arithmetic

Those traits corresponds to basic arithmetic operations, that are guaranteed to give a result, while that one may not be meaningful in some cases.

Those operations can most notably be subject to overflow, meaning the result value of an operation cannot fit into the type on which it is applied.

This trait is mostly useful trough treatments and functions of the [`num` area](https://doc.melodium.tech/latest/en/std/ops/num/index.html).

### `Add`

Type can proceed to addition between two values.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `Sub`

Type can proceed to substraction of two values.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `Mul`

Type can proceed to multiplication of two values.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `Div`

Type can proceed to division of a value by another.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `Rem`

Type can proceed to division of a value by another and give the remainder.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

### `Neg`

Type can negates its values.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `f32` |
| `f64` |

### `Pow`

Type can elevates its values by an exponent.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `Euclid`

Type can proceed to euclidian division of a value by another.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |

## Checked arithmetic

Those traits corresponds to basic arithmetic operations, that may have meaningless result.

Those operations can avoid overflows, as the result value of an operation that cannot fit into the type is ignored.

This trait is mostly useful trough treatments and functions of the [`num` area](https://doc.melodium.tech/latest/en/std/ops/num/index.html).

### `CheckedAdd`

Type can proceed to addition between two values and avoid meaningless results.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `CheckedSub`

Type can proceed to substraction of two values and avoid meaningless results.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `CheckedMul`

Type can proceed to multiplication of two values and avoid meaningless results.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `CheckedDiv`

Type can proceed to division of a value by another and avoid meaningless results.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `CheckedRem`

Type can proceed to division of a value by another and give the remainder, while avoiding meaningless results.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `CheckedNeg`

Type can negates its values and avoid meaningless results.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |

### `CheckedPow`

Type can elevates its values by an exponent and avoid meaningless results.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `CheckedEuclid`

Type can proceed to euclidian division of a value by another and avoid meaningless results.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

## Saturating arithmetic

Those traits corresponds to basic arithmetic operations, that saturates the value to the closest bound to truth if the result cannot fit into the type.

This trait is mostly useful trough treatments and functions of the [`num` area](https://doc.melodium.tech/latest/en/std/ops/num/index.html).

### `SaturatingAdd`

Type can proceed to addition between two values, and bound to minimum or maximum value if addition result is out of range.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `SaturatingSub`

Type can proceed to substraction of two values, and bound to minimum or maximum value if substraction result is out of range.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `SaturatingMul`

Type can proceed to multiplication of two values, and bound to minimum or maximum value if multiplication result is out of range.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

## Wrapping arithmetic

Those traits corresponds to basic arithmetic operations, that wraps on purpose when the operation result goes out of range for the subject type.

This trait is mostly useful trough treatments and functions of the [`num` area](https://doc.melodium.tech/latest/en/std/ops/num/index.html).

### `WrappingAdd`

Type can proceed to addition between two values, and wrap over its value range if addition exceeds type boundaries.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `WrappingSub`

Type can proceed to substraction of two values, and wrap over its value range if substraction exceeds type boundaries.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `WrappingMul`

Type can proceed to multiplication of two values, and wrap over its value range if multiplication exceeds type boundaries.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |

### `WrappingNeg`

Type can negates its values, and wrap over its value range if negation exceeds type boundaries.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |


## `Hash`

Type is subject to [hash](https://en.wikipedia.org/wiki/Hash_function), and so can be used as key to reference data.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `bool` |
| `byte` |
| `char` |
| `string` |

## `Serialize`

Type implements serialization, meaning it can be turned into linear data and send out from a program.

| Implemented by |
|----------------|
| `void` |
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |
| `bool` |
| `byte` |
| `char` |
| `string` |

## `Deserialize`

Type implements deserialization, meaning it can be received from outside of a program and parsed to build a value.

| Implemented by |
|----------------|
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |
| `bool` |
| `byte` |
| `char` |
| `string` |

## `Display`

Type can be rendered as human readable string and possibly displayed to users in a way making sense for them.

> `Display` trait is different to the `ToString` trait as it does not have the same functionnal purpose.
> `ToString` is expected to be a technical conversion of data, while `Display` is meant to expose it to human eyes.

| Implemented by |
|----------------|
| `void` |
| `i8` |
| `i16` |
| `i32` |
| `i64` |
| `i128` |
| `u8` |
| `u16` |
| `u32` |
| `u64` |
| `u128` |
| `f32` |
| `f64` |
| `bool` |
| `byte` |
| `char` |
| `string` |
