use super::super::Data;
use super::Value;
use std::sync::Arc;

/// Trait allowing to get real data based on Rust type.
///
/// This trait exist to circumvent E0119 that is disabling us to use TryInto.
/// See https://github.com/rust-lang/rust/issues/50133
pub trait GetData<T>: Sized {
    fn try_data(self) -> Result<T, ()>;
}

/// Identity extraction: a `Value` handed through unchanged. This is what an unconstrained
/// mel generic (`#[mel_function] generic T ()`, or any bound not restricted to custom
/// `Data` types) actually resolves to at the Rust level — the macro type-aliases every
/// such generic name to `Value` itself — so `Vec<T>`/`Option<T>` involving a bare generic
/// need `Self: GetData<T>` to hold for `T = Value` in order to compose with the existing
/// blanket `Vec`/`Option` impls below, exactly like any other scalar type does.
impl GetData<Value> for Value {
    fn try_data(self) -> Result<Value, ()> {
        Ok(self)
    }
}

impl From<()> for Value {
    fn from(value: ()) -> Self {
        Value::Void(value)
    }
}

impl GetData<()> for Value {
    fn try_data(self) -> Result<(), ()> {
        match self {
            Value::Void(_) => Ok(()),
            _ => Err(()),
        }
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::I8(value)
    }
}

impl GetData<i8> for Value {
    fn try_data(self) -> Result<i8, ()> {
        match self {
            Value::I8(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::I16(value)
    }
}

impl GetData<i16> for Value {
    fn try_data(self) -> Result<i16, ()> {
        match self {
            Value::I16(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::I32(value)
    }
}

impl GetData<i32> for Value {
    fn try_data(self) -> Result<i32, ()> {
        match self {
            Value::I32(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::I64(value)
    }
}

impl GetData<i64> for Value {
    fn try_data(self) -> Result<i64, ()> {
        match self {
            Value::I64(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<i128> for Value {
    fn from(value: i128) -> Self {
        Value::I128(value)
    }
}

impl GetData<i128> for Value {
    fn try_data(self) -> Result<i128, ()> {
        match self {
            Value::I128(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::U8(value)
    }
}

impl GetData<u8> for Value {
    fn try_data(self) -> Result<u8, ()> {
        match self {
            Value::U8(val) => Ok(val),
            Value::Byte(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::U16(value)
    }
}

impl GetData<u16> for Value {
    fn try_data(self) -> Result<u16, ()> {
        match self {
            Value::U16(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::U32(value)
    }
}

impl GetData<u32> for Value {
    fn try_data(self) -> Result<u32, ()> {
        match self {
            Value::U32(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::U64(value)
    }
}

impl GetData<u64> for Value {
    fn try_data(self) -> Result<u64, ()> {
        match self {
            Value::U64(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<u128> for Value {
    fn from(value: u128) -> Self {
        Value::U128(value)
    }
}

impl GetData<u128> for Value {
    fn try_data(self) -> Result<u128, ()> {
        match self {
            Value::U128(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::F32(value)
    }
}

impl GetData<f32> for Value {
    fn try_data(self) -> Result<f32, ()> {
        match self {
            Value::F32(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::F64(value)
    }
}

impl GetData<f64> for Value {
    fn try_data(self) -> Result<f64, ()> {
        match self {
            Value::F64(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl GetData<bool> for Value {
    fn try_data(self) -> Result<bool, ()> {
        match self {
            Value::Bool(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::Char(value)
    }
}

impl GetData<char> for Value {
    fn try_data(self) -> Result<char, ()> {
        match self {
            Value::Char(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl GetData<String> for Value {
    fn try_data(self) -> Result<String, ()> {
        match self {
            Value::String(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        Value::Option(value.map(|val| Box::new(val.into())))
    }
}

impl<T> GetData<Option<T>> for Value
where
    Self: GetData<T>,
{
    fn try_data(self) -> Result<Option<T>, ()> {
        match self {
            Value::Option(val) => {
                if let Some(val) = val {
                    match val.try_data() {
                        Ok(val) => Ok(Some(val)),
                        Err(_) => Err(()),
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Err(()),
        }
    }
}

impl<T: Into<Value> + 'static> From<Vec<T>> for Value {
    fn from(value: Vec<T>) -> Self {
        // `try_from_vec` recognizes `T` at runtime (via `Any`) as one of `PackedArray`'s
        // primitive types and, if so, packs it — automatically, for every caller that
        // uses this conversion (`.into()`), with no per-call-site opt-in needed. Falls
        // through to the ordinary boxed representation for anything else (`String`,
        // `Arc<dyn Data>`, nested `Vec`/`Option`, ...). See ticket #116.
        match super::PackedArray::try_from_vec(value) {
            Ok(packed) => Value::Packed(packed),
            Err(value) => Value::Vec(value.into_iter().map(|val| val.into()).collect()),
        }
    }
}

impl<T: 'static> GetData<Vec<T>> for Value
where
    Self: GetData<T>,
{
    fn try_data(self) -> Result<Vec<T>, ()> {
        match self {
            Value::Vec(val) => {
                let mut result = Vec::with_capacity(val.len());
                for val in val {
                    match val.try_data() {
                        Ok(val) => result.push(val),
                        Err(_) => return Err(()),
                    }
                }
                Ok(result)
            }
            // Direct extraction for a `Value::Packed`: `try_into_vec` recognizes `T` at
            // runtime (via `Any`) as the array's actual element type and, if so, hands
            // back the `Vec<T>` straight from the array's storage — no intermediate
            // `Value` ever created. This covers the common case (`T` one of
            // `PackedArray`'s own primitive types) with zero boxing; callers that can
            // accept `Arc<Vec<T>>` instead should extract through
            // `GetData<Arc<Vec<T>>>` (see `packed.rs`), which stays zero-copy even when
            // the array is shared.
            //
            // If `T` doesn't match the array's stored primitive directly (e.g. `T =
            // Value` itself — what a bare, unconstrained mel generic resolves to, see
            // `GetData<Value> for Value` above), that's not necessarily a genuine type
            // mismatch: falling back to the same expand-then-extract path `Value::Vec`
            // uses above is still correct, just not the zero-copy fast path. Skipping
            // this fallback would make `GetData<Vec<T>>` reject a legitimately-typed
            // `Packed` source whenever `T` isn't one of the 15 packable primitives -
            // exactly the panic/silent-failure bug this whole mechanism exists to avoid.
            Value::Packed(arr) => match arr.try_into_vec() {
                Ok(vec) => Ok(vec),
                Err(arr) => {
                    let val = arr.into_values();
                    let mut result = Vec::with_capacity(val.len());
                    for val in val {
                        match val.try_data() {
                            Ok(val) => result.push(val),
                            Err(_) => return Err(()),
                        }
                    }
                    Ok(result)
                }
            },
            _ => Err(()),
        }
    }
}

impl From<Arc<dyn Data>> for Value {
    fn from(value: Arc<dyn Data>) -> Self {
        Value::Data(value)
    }
}

impl GetData<Arc<dyn Data>> for Value {
    fn try_data(self) -> Result<Arc<dyn Data>, ()> {
        match self {
            Value::Data(val) => Ok(val),
            _ => Err(()),
        }
    }
}

/// Casts straight to a concrete `Data` implementor, folding the two-step
/// `GetData::<Arc<dyn Data>>::try_data(val).unwrap().downcast_arc::<D>().unwrap()` idiom
/// (used throughout `libs/*-mel` for every custom data type) into one call — and, via
/// `recv_one_as`, into one non-panicking `RecvResult`. Coexists with the `Arc<dyn Data>`
/// impl above without conflict: `D` carries an implicit `Sized` bound here, which the
/// unsized `dyn Data` can never satisfy, so the two can never overlap for the same type.
impl<D: Data> GetData<Arc<D>> for Value {
    fn try_data(self) -> Result<Arc<D>, ()> {
        let data: Arc<dyn Data> = GetData::<Arc<dyn Data>>::try_data(self)?;
        data.downcast_arc::<D>().map_err(|_| ())
    }
}

#[cfg(test)]
mod arc_data_getdata_tests {
    use super::*;

    // Type-check only: does the recursive Vec<T>/Option<T> machinery compose with the new
    // Arc<D: Data> impl for free, without any additional impl written for these shapes?
    fn _assert_composes<D: Data>()
    where
        Value: GetData<Vec<Arc<D>>> + GetData<Option<Arc<D>>> + GetData<Vec<Option<Arc<D>>>>,
    {
    }
}

#[cfg(test)]
mod auto_packing_tests {
    use super::*;
    use crate::executive::PackedArray;

    // The whole point of routing `From<Vec<T>>` through `PackedArray::try_from_vec`:
    // an ordinary `.into()` call, exactly what every existing and third-party caller
    // already writes, must produce `Value::Packed` automatically for a packable
    // primitive - no call site needs to know `Packed` exists.
    #[test]
    fn into_produces_packed_for_a_packable_primitive() {
        let value: Value = vec![1u8, 2, 3].into();
        assert!(matches!(value, Value::Packed(PackedArray::U8(_))));
    }

    #[test]
    fn into_still_produces_the_boxed_form_for_a_non_packable_type() {
        let value: Value = vec!["a".to_string(), "b".to_string()].into();
        assert!(matches!(value, Value::Vec(_)));
    }

    // `Value` extracting to itself, trivially - what a bare mel generic (`generic T ()`)
    // actually resolves to at the Rust level, per `melodium-macro`'s typedef codegen.
    #[test]
    fn value_extracts_to_itself() {
        let value = Value::U64(42);
        let extracted: Value = value.clone().try_data().unwrap();
        assert_eq!(extracted, value);
    }

    // The exact bug this fallback exists to close: a `#[mel_function]` with an
    // unconstrained generic `Vec<T>` parameter (e.g. `contains(vector: Vec<T>, ...)`)
    // extracts via `GetData::<Vec<Value>>::try_data`, since `T` is type-aliased to
    // `Value` for a bare generic. Before the fallback, `PackedArray::try_into_vec::<Value>`
    // would always fail (no primitive is ever `Value` itself) and the whole extraction
    // would incorrectly reject a legitimately `Value::Packed` source.
    #[test]
    fn vec_of_value_extracts_correctly_from_a_packed_source() {
        let value = Value::Packed(PackedArray::I64(Arc::new(vec![1, 2, 3])));
        let extracted: Vec<Value> = value.try_data().unwrap();
        assert_eq!(extracted, vec![Value::I64(1), Value::I64(2), Value::I64(3)]);
    }

    #[test]
    fn vec_of_value_still_extracts_correctly_from_a_boxed_source() {
        let value = Value::Vec(vec![Value::I64(1), Value::I64(2)]);
        let extracted: Vec<Value> = value.try_data().unwrap();
        assert_eq!(extracted, vec![Value::I64(1), Value::I64(2)]);
    }

    // `From<Option<T>>` composes through `.into()` too, so this must auto-pack exactly
    // like the bare `Vec<u8>` case, with no extra code needed for the nested shape.
    #[test]
    fn into_auto_packs_through_nested_option() {
        let value: Value = Some(vec![1u8, 2, 3]).into();
        match value {
            Value::Option(Some(inner)) => {
                assert!(matches!(*inner, Value::Packed(PackedArray::U8(_))));
            }
            other => panic!(
                "expected Value::Option(Some(Value::Packed(_))), got {:?}",
                other
            ),
        }
    }

    // Round-trips: what auto-packing constructs, the existing extraction path (whether
    // the fast `Arc<Vec<T>>` route or the generic `Vec<T>` route) must read back.
    #[test]
    fn auto_packed_value_extracts_correctly_both_ways() {
        let value: Value = vec![1u8, 2, 3].into();

        let as_vec: Vec<u8> = value.clone().try_data().unwrap();
        assert_eq!(as_vec, vec![1, 2, 3]);

        let as_arc: Arc<Vec<u8>> = value.try_data().unwrap();
        assert_eq!(*as_arc, vec![1, 2, 3]);
    }
}
