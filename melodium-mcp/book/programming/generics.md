# Generics

Generics are a mechanism allowing to rely on type abstraction to process data.

```mel
use std/ops/num::isPositive
use std/conv::saturatingToI64

treatment demonstration()
  input  floating_point_value: Stream<f32>
  output integer_value: Stream<i64>
  output is_positive: Stream<bool>
{
    // Treatments isPositive and saturatingToI64 are depending on generic type,
    // that have to be given at instanciation.
    isPositive<f32>()
    saturatingToI64<f32>()

    Self.floating_point_value ------> isPositive.value,positive --> Self.is_positive
    Self.floating_point_value -> saturatingToI64.value,into ------> Self.integer_value
}
```
_Reference for [isPositive](https://doc.melodium.tech/latest/en/std/ops/num/isPositive.html), [saturatingToI64](https://doc.melodium.tech/latest/en/std/conv/saturatingToI64.html)_

![demonstration graph](images/generics_demonstration.png)  

## Traits restriction

Instead of allowing any kind of data, generics can be restrained to specific traits, requiring the given data type to implement these traits to be used with the element.

```mel
use std/ops/num::isPositive
use std/conv::saturatingToI64

// A treatment can have generic parameter, with optionnal trait restrictions,
// in order to fit its functionnal abilities.
treatment demonstration<N: Float + SaturatingToI64>()
  // Generics can be used at any place a type can be given.
  input  floating_point_value: Stream<N>
  output integer_value: Stream<i64>
  output is_positive: Stream<bool>
{
    // As N fits the Float and SaturatingToI64 traits,
    // it can be passed to those treatments too.
    isPositive<N>()
    saturatingToI64<N>()

    Self.floating_point_value ------> isPositive.value,positive --> Self.is_positive
    Self.floating_point_value -> saturatingToI64.value,into ------> Self.integer_value
}
```