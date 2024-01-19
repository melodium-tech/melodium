use proc_macro2::TokenStream;
use quote::quote;

pub fn object_traits<T: AsRef<str> + PartialEq<str>>(
    name_str: &str,
    traits: &Vec<T>,
) -> TokenStream {
    let name: TokenStream = name_str.parse().unwrap();

    let to_i8 = to_i8(name_str, traits.iter().any(|t| t == "ToI8"));
    let to_i16 = to_i16(name_str, traits.iter().any(|t| t == "ToI16"));
    let to_i32 = to_i32(name_str, traits.iter().any(|t| t == "ToI32"));
    let to_i64 = to_i64(name_str, traits.iter().any(|t| t == "ToI64"));
    let to_i128 = to_i128(name_str, traits.iter().any(|t| t == "ToI128"));
    let to_u8 = to_u8(name_str, traits.iter().any(|t| t == "ToU8"));
    let to_u16 = to_u16(name_str, traits.iter().any(|t| t == "ToU16"));
    let to_u32 = to_u32(name_str, traits.iter().any(|t| t == "ToU32"));
    let to_u64 = to_u64(name_str, traits.iter().any(|t| t == "ToU64"));
    let to_u128 = to_u128(name_str, traits.iter().any(|t| t == "ToU128"));
    let to_f32 = to_f32(name_str, traits.iter().any(|t| t == "ToF32"));
    let to_f64 = to_f64(name_str, traits.iter().any(|t| t == "ToF64"));
    let to_bool = to_bool(name_str, traits.iter().any(|t| t == "ToBool"));
    let to_byte = to_byte(name_str, traits.iter().any(|t| t == "ToByte"));
    let to_char = to_char(name_str, traits.iter().any(|t| t == "ToChar"));
    let to_string = to_string(name_str, traits.iter().any(|t| t == "ToString"));
    let try_to_i8 = try_to_i8(name_str, traits.iter().any(|t| t == "TryToI8"));
    let try_to_i16 = try_to_i16(name_str, traits.iter().any(|t| t == "TryToI16"));
    let try_to_i32 = try_to_i32(name_str, traits.iter().any(|t| t == "TryToI32"));
    let try_to_i64 = try_to_i64(name_str, traits.iter().any(|t| t == "TryToI64"));
    let try_to_i128 = try_to_i128(name_str, traits.iter().any(|t| t == "TryToI128"));
    let try_to_u8 = try_to_u8(name_str, traits.iter().any(|t| t == "TryToU8"));
    let try_to_u16 = try_to_u16(name_str, traits.iter().any(|t| t == "TryToU16"));
    let try_to_u32 = try_to_u32(name_str, traits.iter().any(|t| t == "TryToU32"));
    let try_to_u64 = try_to_u64(name_str, traits.iter().any(|t| t == "TryToU64"));
    let try_to_u128 = try_to_u128(name_str, traits.iter().any(|t| t == "TryToU128"));
    let try_to_f32 = try_to_f32(name_str, traits.iter().any(|t| t == "TryToF32"));
    let try_to_f64 = try_to_f64(name_str, traits.iter().any(|t| t == "TryToF64"));
    let try_to_bool = try_to_bool(name_str, traits.iter().any(|t| t == "TryToBool"));
    let try_to_byte = try_to_byte(name_str, traits.iter().any(|t| t == "TryToByte"));
    let try_to_char = try_to_char(name_str, traits.iter().any(|t| t == "TryToChar"));
    let try_to_string = try_to_string(name_str, traits.iter().any(|t| t == "TryToString"));
    let saturating_to_i8 = saturating_to_i8(name_str, traits.iter().any(|t| t == "SaturatingToI8"));
    let saturating_to_i16 =
        saturating_to_i16(name_str, traits.iter().any(|t| t == "SaturatingToI16"));
    let saturating_to_i32 =
        saturating_to_i32(name_str, traits.iter().any(|t| t == "SaturatingToI32"));
    let saturating_to_i64 =
        saturating_to_i64(name_str, traits.iter().any(|t| t == "SaturatingToI64"));
    let saturating_to_i128 =
        saturating_to_i128(name_str, traits.iter().any(|t| t == "SaturatingToI128"));
    let saturating_to_u8 = saturating_to_u8(name_str, traits.iter().any(|t| t == "SaturatingToU8"));
    let saturating_to_u16 =
        saturating_to_u16(name_str, traits.iter().any(|t| t == "SaturatingToU16"));
    let saturating_to_u32 =
        saturating_to_u32(name_str, traits.iter().any(|t| t == "SaturatingToU32"));
    let saturating_to_u64 =
        saturating_to_u64(name_str, traits.iter().any(|t| t == "SaturatingToU64"));
    let saturating_to_u128 =
        saturating_to_u128(name_str, traits.iter().any(|t| t == "SaturatingToU128"));
    let saturating_to_f32 =
        saturating_to_f32(name_str, traits.iter().any(|t| t == "SaturatingToF32"));
    let saturating_to_f64 =
        saturating_to_f64(name_str, traits.iter().any(|t| t == "SaturatingToF64"));
    let binary = binary(name_str, traits.iter().any(|t| t == "Binary"));
    let signed = signed(name_str, traits.iter().any(|t| t == "Signed"));
    let float = float(name_str, traits.iter().any(|t| t == "Float"));
    let partial_equality =
        partial_equality(name_str, traits.iter().any(|t| t == "PartialEquality"));
    let partial_order = partial_order(name_str, traits.iter().any(|t| t == "PartialOrder"));
    let order = order(name_str, traits.iter().any(|t| t == "Order"));
    let add = add(name_str, traits.iter().any(|t| t == "Add"));
    let checked_add = checked_add(name_str, traits.iter().any(|t| t == "CheckedAdd"));
    let saturating_add = saturating_add(name_str, traits.iter().any(|t| t == "SaturatingAdd"));
    let wrapping_add = wrapping_add(name_str, traits.iter().any(|t| t == "WrappingAdd"));
    let sub = sub(name_str, traits.iter().any(|t| t == "Sub"));
    let checked_sub = checked_sub(name_str, traits.iter().any(|t| t == "CheckedSub"));
    let saturating_sub = saturating_sub(name_str, traits.iter().any(|t| t == "SaturatingSub"));
    let wrapping_sub = wrapping_sub(name_str, traits.iter().any(|t| t == "WrappingSub"));
    let mul = mul(name_str, traits.iter().any(|t| t == "Mul"));
    let checked_mul = checked_mul(name_str, traits.iter().any(|t| t == "CheckedMul"));
    let saturating_mul = saturating_mul(name_str, traits.iter().any(|t| t == "SaturatingMul"));
    let wrapping_mul = wrapping_mul(name_str, traits.iter().any(|t| t == "WrappingMul"));
    let div = div(name_str, traits.iter().any(|t| t == "Div"));
    let checked_div = checked_div(name_str, traits.iter().any(|t| t == "CheckedDiv"));
    let rem = rem(name_str, traits.iter().any(|t| t == "Rem"));
    let checked_rem = checked_rem(name_str, traits.iter().any(|t| t == "CheckedRem"));
    let neg = neg(name_str, traits.iter().any(|t| t == "Neg"));
    let checked_neg = checked_neg(name_str, traits.iter().any(|t| t == "CheckedNeg"));
    let wrapping_neg = wrapping_neg(name_str, traits.iter().any(|t| t == "WrappingNeg"));
    let pow = pow(name_str, traits.iter().any(|t| t == "Pow"));
    let checked_pow = checked_pow(name_str, traits.iter().any(|t| t == "CheckedPow"));
    let euclid = euclid(name_str, traits.iter().any(|t| t == "Euclid"));
    let checked_euclid = checked_euclid(name_str, traits.iter().any(|t| t == "CheckedEuclid"));

    quote! {

        impl melodium_core::common::executive::DataTrait for #name {
            #to_i8
            #to_i16
            #to_i32
            #to_i64
            #to_i128
            #to_u8
            #to_u16
            #to_u32
            #to_u64
            #to_u128
            #to_f32
            #to_f64
            #to_bool
            #to_byte
            #to_char
            #to_string
            #try_to_i8
            #try_to_i16
            #try_to_i32
            #try_to_i64
            #try_to_i128
            #try_to_u8
            #try_to_u16
            #try_to_u32
            #try_to_u64
            #try_to_u128
            #try_to_f32
            #try_to_f64
            #try_to_bool
            #try_to_byte
            #try_to_char
            #try_to_string
            #saturating_to_i8
            #saturating_to_i16
            #saturating_to_i32
            #saturating_to_i64
            #saturating_to_i128
            #saturating_to_u8
            #saturating_to_u16
            #saturating_to_u32
            #saturating_to_u64
            #saturating_to_u128
            #saturating_to_f32
            #saturating_to_f64

            #binary

            #signed

            #float

            #partial_equality

            #partial_order

            #order

            #add
            #checked_add
            #saturating_add
            #wrapping_add
            #sub
            #checked_sub
            #saturating_sub
            #wrapping_sub
            #mul
            #checked_mul
            #saturating_mul
            #wrapping_mul
            #div
            #checked_div
            #rem
            #checked_rem
            #neg
            #checked_neg
            #wrapping_neg
            #pow
            #checked_pow

            #euclid
            #checked_euclid
        }

    }
}

fn to_i8(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_i8(&self) -> i8 {
                melodium_core::executive::ToI8::to_i8(self)
            }
        }
    } else {
        quote! {
            fn to_i8(&self) -> i8 {
                panic!("ToI8 not implemented for {}", #name)
            }
        }
    }
}
fn to_i16(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_i16(&self) -> i16 {
                melodium_core::executive::ToI16::to_i16(self)
            }
        }
    } else {
        quote! {
            fn to_i16(&self) -> i16 {
                panic!("ToI16 not implemented for {}", #name)
            }
        }
    }
}
fn to_i32(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_i32(&self) -> i32 {
                melodium_core::executive::ToI32::to_i32(self)
            }
        }
    } else {
        quote! {
            fn to_i32(&self) -> i32 {
                panic!("ToI32 not implemented for {}", #name)
            }
        }
    }
}
fn to_i64(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_i64(&self) -> i64 {
                melodium_core::executive::ToI64::to_i64(self)
            }
        }
    } else {
        quote! {
            fn to_i64(&self) -> i64 {
                panic!("ToI64 not implemented for {}", #name)
            }
        }
    }
}
fn to_i128(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_i128(&self) -> i128 {
                melodium_core::executive::ToI128::to_i128(self)
            }
        }
    } else {
        quote! {
            fn to_i128(&self) -> i128 {
                panic!("ToI128 not implemented for {}", #name)
            }
        }
    }
}
fn to_u8(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_u8(&self) -> u8 {
                melodium_core::executive::ToU8::to_u8(self)
            }
        }
    } else {
        quote! {
            fn to_u8(&self) -> u8 {
                panic!("ToU8 not implemented for {}", #name)
            }
        }
    }
}
fn to_u16(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_u16(&self) -> u16 {
                melodium_core::executive::ToU16::to_u16(self)
            }
        }
    } else {
        quote! {
            fn to_u16(&self) -> u16 {
                panic!("ToU16 not implemented for {}", #name)
            }
        }
    }
}
fn to_u32(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_u32(&self) -> u32 {
                melodium_core::executive::ToU32::to_u32(self)
            }
        }
    } else {
        quote! {
            fn to_u32(&self) -> u32 {
                panic!("ToU32 not implemented for {}", #name)
            }
        }
    }
}
fn to_u64(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_u64(&self) -> u64 {
                melodium_core::executive::ToU64::to_u64(self)
            }
        }
    } else {
        quote! {
            fn to_u64(&self) -> u64 {
                panic!("ToU64 not implemented for {}", #name)
            }
        }
    }
}
fn to_u128(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_u128(&self) -> u128 {
                melodium_core::executive::ToU128::to_u128(self)
            }
        }
    } else {
        quote! {
            fn to_u128(&self) -> u128 {
                panic!("ToU128 not implemented for {}", #name)
            }
        }
    }
}
fn to_f32(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_f32(&self) -> f32 {
                melodium_core::executive::ToF32::to_f32(self)
            }
        }
    } else {
        quote! {
            fn to_f32(&self) -> f32 {
                panic!("ToF32 not implemented for {}", #name)
            }
        }
    }
}
fn to_f64(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_f64(&self) -> f64 {
                melodium_core::executive::ToF64::to_f64(self)
            }
        }
    } else {
        quote! {
            fn to_f64(&self) -> f64 {
                panic!("ToF64 not implemented for {}", #name)
            }
        }
    }
}
fn to_bool(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_bool(&self) -> bool {
                melodium_core::executive::ToBool::to_bool(self)
            }
        }
    } else {
        quote! {
            fn to_bool(&self) -> bool {
                panic!("ToBool not implemented for {}", #name)
            }
        }
    }
}
fn to_byte(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_byte(&self) -> u8 {
                melodium_core::executive::ToByte::to_byte(self)
            }
        }
    } else {
        quote! {
            fn to_byte(&self) -> u8 {
                panic!("ToByte not implemented for {}", #name)
            }
        }
    }
}
fn to_char(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_char(&self) -> char {
                melodium_core::executive::ToChar::to_char(self)
            }
        }
    } else {
        quote! {
            fn to_char(&self) -> char {
                panic!("ToChar not implemented for {}", #name)
            }
        }
    }
}
fn to_string(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_string(&self) -> String {
                melodium_core::executive::ToString::to_string(self)
            }
        }
    } else {
        quote! {
            fn to_string(&self) -> String {
                panic!("ToString not implemented for {}", #name)
            }
        }
    }
}
fn try_to_i8(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_i8(&self) -> Option<i8> {
                melodium_core::executive::TryToI8::try_to_i8(self)
            }
        }
    } else {
        quote! {
            fn try_to_i8(&self) -> Option<i8> {
                panic!("TryToI8 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_i16(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_i16(&self) -> Option<i16> {
                melodium_core::executive::TryToI16::try_to_i16(self)
            }
        }
    } else {
        quote! {
            fn try_to_i16(&self) -> Option<i16> {
                panic!("TryToI16 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_i32(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_i32(&self) -> Option<i32> {
                melodium_core::executive::TryToI32::try_to_i32(self)
            }
        }
    } else {
        quote! {
            fn try_to_i32(&self) -> Option<i32> {
                panic!("TryToI32 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_i64(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_i64(&self) -> Option<i64> {
                melodium_core::executive::TryToI64::try_to_i64(self)
            }
        }
    } else {
        quote! {
            fn try_to_i64(&self) -> Option<i64> {
                panic!("TryToI64 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_i128(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_i128(&self) -> Option<i128> {
                melodium_core::executive::TryToI128::try_to_i128(self)
            }
        }
    } else {
        quote! {
            fn try_to_i128(&self) -> Option<i128> {
                panic!("TryToI128 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_u8(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_u8(&self) -> Option<u8> {
                melodium_core::executive::TryToU8::try_to_u8(self)
            }
        }
    } else {
        quote! {
            fn try_to_u8(&self) -> Option<u8> {
                panic!("TryToU8 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_u16(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_u16(&self) -> Option<u16> {
                melodium_core::executive::TryToU16::try_to_u16(self)
            }
        }
    } else {
        quote! {
            fn try_to_u16(&self) -> Option<u16> {
                panic!("TryToU16 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_u32(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_u32(&self) -> Option<u32> {
                melodium_core::executive::TryToU32::try_to_u32(self)
            }
        }
    } else {
        quote! {
            fn try_to_u32(&self) -> Option<u32> {
                panic!("TryToU32 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_u64(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_u64(&self) -> Option<u64> {
                melodium_core::executive::TryToU64::try_to_u64(self)
            }
        }
    } else {
        quote! {
            fn try_to_u64(&self) -> Option<u64> {
                panic!("TryToU64 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_u128(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_u128(&self) -> Option<u128> {
                melodium_core::executive::TryToU128::try_to_u128(self)
            }
        }
    } else {
        quote! {
            fn try_to_u128(&self) -> Option<u128> {
                panic!("TryToU128 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_f32(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_f32(&self) -> Option<f32> {
                melodium_core::executive::TryToF32::try_to_f32(self)
            }
        }
    } else {
        quote! {
            fn try_to_f32(&self) -> Option<f32> {
                panic!("TryToF32 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_f64(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_f64(&self) -> Option<f64> {
                melodium_core::executive::TryToF64::try_to_f64(self)
            }
        }
    } else {
        quote! {
            fn try_to_f64(&self) -> Option<f64> {
                panic!("TryToF64 not implemented for {}", #name)
            }
        }
    }
}
fn try_to_bool(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_bool(&self) -> Option<bool> {
                melodium_core::executive::TryToBool::try_to_bool(self)
            }
        }
    } else {
        quote! {
            fn try_to_bool(&self) -> Option<bool> {
                panic!("TryToBool not implemented for {}", #name)
            }
        }
    }
}
fn try_to_byte(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_byte(&self) -> Option<u8> {
                melodium_core::executive::TryToByte::try_to_byte(self)
            }
        }
    } else {
        quote! {
            fn try_to_byte(&self) -> Option<u8> {
                panic!("TryToByte not implemented for {}", #name)
            }
        }
    }
}
fn try_to_char(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_char(&self) -> Option<char> {
                melodium_core::executive::TryToChar::try_to_char(self)
            }
        }
    } else {
        quote! {
            fn try_to_char(&self) -> Option<char> {
                panic!("TryToChar not implemented for {}", #name)
            }
        }
    }
}
fn try_to_string(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn try_to_string(&self) -> Option<String> {
                melodium_core::executive::TryToString::try_to_string(self)
            }
        }
    } else {
        quote! {
            fn try_to_string(&self) -> Option<String> {
                panic!("TryToString not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_i8(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_i8(&self) -> i8 {
                melodium_core::executive::SaturatingToI8::saturating_to_i8(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_i8(&self) -> i8 {
                panic!("SaturatingToI8 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_i16(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_i16(&self) -> i16 {
                melodium_core::executive::SaturatingToI16::saturating_to_i16(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_i16(&self) -> i16 {
                panic!("SaturatingToI16 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_i32(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_i32(&self) -> i32 {
                melodium_core::executive::SaturatingToI32::saturating_to_i32(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_i32(&self) -> i32 {
                panic!("SaturatingToI32 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_i64(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_i64(&self) -> i64 {
                melodium_core::executive::SaturatingToI64::saturating_to_i64(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_i64(&self) -> i64 {
                panic!("SaturatingToI64 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_i128(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_i128(&self) -> i128 {
                melodium_core::executive::SaturatingToI128::saturating_to_i128(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_i128(&self) -> i128 {
                panic!("SaturatingToI128 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_u8(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_u8(&self) -> u8 {
                melodium_core::executive::SaturatingToU8::saturating_to_u8(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_u8(&self) -> u8 {
                panic!("SaturatingToU8 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_u16(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_u16(&self) -> u16 {
                melodium_core::executive::SaturatingToU16::saturating_to_u16(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_u16(&self) -> u16 {
                panic!("SaturatingToU16 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_u32(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_u32(&self) -> u32 {
                melodium_core::executive::SaturatingToU32::saturating_to_u32(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_u32(&self) -> u32 {
                panic!("SaturatingToU32 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_u64(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_u64(&self) -> u64 {
                melodium_core::executive::SaturatingToU64::saturating_to_u64(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_u64(&self) -> u64 {
                panic!("SaturatingToU64 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_u128(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_u128(&self) -> u128 {
                melodium_core::executive::SaturatingToU128::saturating_to_u128(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_u128(&self) -> u128 {
                panic!("SaturatingToU128 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_f32(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_f32(&self) -> f32 {
                melodium_core::executive::SaturatingToF32::saturating_to_f32(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_f32(&self) -> f32 {
                panic!("SaturatingToF32 not implemented for {}", #name)
            }
        }
    }
}
fn saturating_to_f64(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_to_f64(&self) -> f64 {
                melodium_core::executive::SaturatingToF64::saturating_to_f64(self)
            }
        }
    } else {
        quote! {
            fn saturating_to_f64(&self) -> f64 {
                panic!("SaturatingToF64 not implemented for {}", #name)
            }
        }
    }
}

fn binary(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn binary_and(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Binary::and(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn binary_or(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Binary::or(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn binary_xor(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Binary::xor(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn binary_not(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Binary::not(self))
                )
            }
        }
    } else {
        quote! {
            fn binary_and(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Binary not implemented for {}", #name)
            }

            fn binary_or(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Binary not implemented for {}", #name)
            }

            fn binary_xor(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Binary not implemented for {}", #name)
            }

            fn binary_not(&self) -> melodium_core::common::executive::Value {
                panic!("Binary not implemented for {}", #name)
            }
        }
    }
}

fn signed(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn signed_abs(&self) -> Option<melodium_core::common::executive::Value> {
                melodium_core::executive::Signed::abs(self)
                    .map(|obj| melodium_core::common::executive::Value::Object(
                        std::sync::Arc::new(obj)
                    ))
            }

            fn signed_signum(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Signed::signum(self))
                )
            }

            fn signed_is_positive(&self) -> bool {
                melodium_core::executive::Signed::is_positive(self)
            }

            fn signed_is_negative(&self) -> bool {
                melodium_core::executive::Signed::is_negative(self)
            }
        }
    } else {
        quote! {
            fn signed_abs(&self) -> Option<melodium_core::common::executive::Value> {
                panic!("Signed not implemented for {}", #name)
            }

            fn signed_signum(&self) -> melodium_core::common::executive::Value {
                panic!("Signed not implemented for {}", #name)
            }

            fn signed_is_positive(&self) -> bool {
                panic!("Signed not implemented for {}", #name)
            }

            fn signed_is_negative(&self) -> bool {
                panic!("Signed not implemented for {}", #name)
            }
        }
    }
}

fn float(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn float_is_nan(&self) -> bool {
                melodium_core::executive::Float::is_nan(self)
            }

            fn float_is_infinite(&self) -> bool {
                melodium_core::executive::Float::is_infinite(self)
            }

            fn float_is_finite(&self) -> bool {
                melodium_core::executive::Float::is_finite(self)
            }

            fn float_is_normal(&self) -> bool {
                melodium_core::executive::Float::is_normal(self)
            }

            fn float_is_subnormal(&self) -> bool {
                melodium_core::executive::Float::is_subnormal(self)
            }

            fn float_floor(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::floor(self))
                )
            }

            fn float_ceil(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::ceil(self))
                )
            }

            fn float_round(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::round(self))
                )
            }

            fn float_trunc(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::trunc(self))
                )
            }

            fn float_fract(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::fract(self))
                )
            }

            fn float_recip(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::recip(self))
                )
            }

            fn float_pow(&self, n: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match n {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Float::pow(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn float_sqrt(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::sqrt(self))
                )
            }

            fn float_exp(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::exp(self))
                )
            }

            fn float_exp2(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::exp2(self))
                )
            }

            fn float_ln(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::ln(self))
                )
            }

            fn float_log(&self, base: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match base {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Float::log(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn float_log2(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::log2(self))
                )
            }

            fn float_log10(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::log10(self))
                )
            }

            fn float_cbrt(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::cbrt(self))
                )
            }

            fn float_hypot(&self, n: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match n {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Float::hypot(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn float_sin(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::sin(self))
                )
            }

            fn float_cos(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::cos(self))
                )
            }

            fn float_tan(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::tan(self))
                )
            }

            fn float_asin(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::asin(self))
                )
            }

            fn float_acos(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::acos(self))
                )
            }

            fn float_atan(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::atan(self))
                )
            }

            fn float_atan2(&self, n: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match n {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Float::atan2(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn float_sinh(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::sinh(self))
                )
            }

            fn float_cosh(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::cosh(self))
                )
            }

            fn float_tanh(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::tanh(self))
                )
            }

            fn float_asinh(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::asinh(self))
                )
            }

            fn float_acosh(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::acosh(self))
                )
            }

            fn float_atanh(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::atanh(self))
                )
            }

            fn float_to_degrees(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::to_degrees(self))
                )
            }

            fn float_to_radians(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Float::to_radians(self))
                )
            }
        }
    } else {
        quote! {
            fn float_is_nan(&self) -> bool {
                panic!("Float not implemented for {}", #name)
            }

            fn float_is_infinite(&self) -> bool {
                panic!("Float not implemented for {}", #name)
            }

            fn float_is_finite(&self) -> bool {
                panic!("Float not implemented for {}", #name)
            }

            fn float_is_normal(&self) -> bool {
                panic!("Float not implemented for {}", #name)
            }

            fn float_is_subnormal(&self) -> bool {
                panic!("Float not implemented for {}", #name)
            }

            fn float_floor(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_ceil(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_round(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_trunc(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_fract(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_recip(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_pow(&self, n: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_sqrt(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_exp(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_exp2(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_ln(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_log(&self, base: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_log2(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_log10(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_cbrt(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_hypot(&self, n: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_sin(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_cos(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_tan(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_asin(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_acos(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_atan(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_atan2(&self, n: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_sinh(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_cosh(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_tanh(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_asinh(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_acosh(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_atanh(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_to_degrees(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }

            fn float_to_radians(&self) -> melodium_core::common::executive::Value {
                panic!("Float not implemented for {}", #name)
            }
        }
    }
}

fn partial_equality(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn partial_equality_eq(&self, other: &melodium_core::common::executive::Value) -> bool {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::PartialEquality::eq(self, &obj)
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn partial_equality_ne(&self, other: &melodium_core::common::executive::Value) -> bool {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::PartialEquality::ne(self, &obj)
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }
        }
    } else {
        quote! {
            fn partial_equality_eq(&self, other: &melodium_core::common::executive::Value) -> bool {
                panic!("PartialEquality not implemented for {}", #name)
            }

            fn partial_equality_ne(&self, other: &melodium_core::common::executive::Value) -> bool {
                panic!("PartialEquality not implemented for {}", #name)
            }
        }
    }
}

fn partial_order(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn partial_order_lt(&self, other: &melodium_core::common::executive::Value) -> bool {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::PartialOrder::lt(self, &obj)
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn partial_order_le(&self, other: &melodium_core::common::executive::Value) -> bool {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::PartialOrder::le(self, &obj)
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn partial_order_gt(&self, other: &melodium_core::common::executive::Value) -> bool {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::PartialOrder::gt(self, &obj)
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn partial_order_ge(&self, other: &melodium_core::common::executive::Value) -> bool {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::PartialOrder::ge(self, &obj)
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }
        }
    } else {
        quote! {
            fn partial_order_lt(&self, other: &melodium_core::common::executive::Value) -> bool {
                panic!("PartialOrder not implemented for {}", #name)
            }

            fn partial_order_le(&self, other: &melodium_core::common::executive::Value) -> bool {
                panic!("PartialOrder not implemented for {}", #name)
            }

            fn partial_order_gt(&self, other: &melodium_core::common::executive::Value) -> bool {
                panic!("PartialOrder not implemented for {}", #name)
            }

            fn partial_order_ge(&self, other: &melodium_core::common::executive::Value) -> bool {
                panic!("PartialOrder not implemented for {}", #name)
            }
        }
    }
}

fn order(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn order_max(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Order::max(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn order_min(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Order::min(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn order_clamp(&self, min: &melodium_core::common::executive::Value, max: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match (min, max) {
                    (melodium_core::common::executive::Value::Object(min), melodium_core::common::executive::Value::Object(max)) =>
                        if let (Ok(min), Ok(max)) = (std::sync::Arc::clone(min).downcast_arc::<Self>(), std::sync::Arc::clone(max).downcast_arc::<Self>()) {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Order::clamp(self, &min, &max))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }
        }
    } else {
        quote! {
            fn order_max(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Order not implemented for {}", #name)
            }

            fn order_min(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Order not implemented for {}", #name)
            }

            fn order_clamp(&self, min: &melodium_core::common::executive::Value, max: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Order not implemented for {}", #name)
            }
        }
    }
}

fn add(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn add(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Add::add(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn add(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Add not implemented for {}", #name)
            }
        }
    }
}

fn checked_add(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn checked_add(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::CheckedAdd::checked_add(self, &obj)
                                .map(|obj| melodium_core::common::executive::Value::Object(
                                    std::sync::Arc::new(obj)
                                ))
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }
        }
    } else {
        quote! {
            fn checked_add(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                panic!("CheckedAdd not implemented for {}", #name)
            }
        }
    }
}

fn saturating_add(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_add(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::SaturatingAdd::saturating_add(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn saturating_add(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("SaturatingAdd not implemented for {}", #name)
            }
        }
    }
}
fn wrapping_add(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn wrapping_add(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::WrappingAdd::wrapping_add(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn wrapping_add(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("WrappingAdd not implemented for {}", #name)
            }
        }
    }
}
fn sub(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn sub(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Sub::sub(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn sub(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Sub not implemented for {}", #name)
            }
        }
    }
}
fn checked_sub(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn checked_sub(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::CheckedSub::checked_sub(self, &obj)
                                .map(|obj| melodium_core::common::executive::Value::Object(
                                    std::sync::Arc::new(obj)
                                ))
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }
        }
    } else {
        quote! {
            fn checked_sub(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                panic!("CheckedSub not implemented for {}", #name)
            }
        }
    }
}
fn saturating_sub(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_sub(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::SaturatingSub::saturating_sub(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn saturating_sub(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("SaturatingSub not implemented for {}", #name)
            }
        }
    }
}
fn wrapping_sub(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn wrapping_sub(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::WrappingSub::wrapping_sub(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn wrapping_sub(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("WrappingSub not implemented for {}", #name)
            }
        }
    }
}
fn mul(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn mul(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Mul::mul(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn mul(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Mul not implemented for {}", #name)
            }
        }
    }
}
fn checked_mul(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn checked_mul(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::CheckedMul::checked_mul(self, &obj)
                                .map(|obj| melodium_core::common::executive::Value::Object(
                                    std::sync::Arc::new(obj)
                                ))
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }
        }
    } else {
        quote! {
            fn checked_mul(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                panic!("CheckedMul not implemented for {}", #name)
            }
        }
    }
}
fn saturating_mul(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn saturating_mul(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::SaturatingMul::saturating_mul(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn saturating_mul(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("SaturatingMul not implemented for {}", #name)
            }
        }
    }
}
fn wrapping_mul(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn wrapping_mul(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::WrappingMul::wrapping_mul(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn wrapping_mul(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("WrappingMul not implemented for {}", #name)
            }
        }
    }
}
fn div(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn div(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Div::div(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn div(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Div not implemented for {}", #name)
            }
        }
    }
}
fn checked_div(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn checked_div(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::CheckedDiv::checked_div(self, &obj)
                                .map(|obj| melodium_core::common::executive::Value::Object(
                                    std::sync::Arc::new(obj)
                                ))
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }
        }
    } else {
        quote! {
            fn checked_div(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                panic!("CheckedDiv not implemented for {}", #name)
            }
        }
    }
}
fn rem(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn rem(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Rem::rem(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn rem(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Rem not implemented for {}", #name)
            }
        }
    }
}
fn checked_rem(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn checked_rem(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::CheckedRem::checked_rem(self, &obj)
                                .map(|obj| melodium_core::common::executive::Value::Object(
                                    std::sync::Arc::new(obj)
                                ))
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }
        }
    } else {
        quote! {
            fn checked_rem(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                panic!("CheckedRem not implemented for {}", #name)
            }
        }
    }
}

fn neg(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn neg(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Neg::neg(self))
                )
            }
        }
    } else {
        quote! {
            fn neg(&self) -> melodium_core::common::executive::Value {
                panic!("Neg not implemented for {}", #name)
            }
        }
    }
}

fn checked_neg(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn checked_neg(&self) -> Option<melodium_core::common::executive::Value> {
                melodium_core::executive::CheckedNeg::checked_neg(self)
                    .map(|obj| melodium_core::common::executive::Value::Object(
                        std::sync::Arc::new(obj)
                    ))
            }
        }
    } else {
        quote! {
            fn checked_neg(&self) -> Option<melodium_core::common::executive::Value> {
                panic!("CheckedNeg not implemented for {}", #name)
            }
        }
    }
}

fn wrapping_neg(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn wrapping_neg(&self) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::WrappingNeg::wrapping_neg(self))
                )
            }
        }
    } else {
        quote! {
            fn wrapping_neg(&self) -> melodium_core::common::executive::Value {
                panic!("WrappingNeg not implemented for {}", #name)
            }
        }
    }
}

fn pow(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn pow(&self, exp: &u32) -> melodium_core::common::executive::Value {
                melodium_core::common::executive::Value::Object(
                    std::sync::Arc::new(melodium_core::executive::Pow::pow(self, exp))
                )
            }
        }
    } else {
        quote! {
            fn pow(&self, exp: &u32) -> melodium_core::common::executive::Value {
                panic!("Pow not implemented for {}", #name)
            }
        }
    }
}

fn checked_pow(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn checked_pow(&self, exp: &u32) -> Option<melodium_core::common::executive::Value> {
                melodium_core::executive::CheckedPow::checked_pow(self, exp)
                    .map(|obj| melodium_core::common::executive::Value::Object(
                        std::sync::Arc::new(obj)
                    ))
            }
        }
    } else {
        quote! {
            fn checked_pow(&self, exp: &u32) -> Option<melodium_core::common::executive::Value> {
                panic!("CheckedPow not implemented for {}", #name)
            }
        }
    }
}

fn euclid(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn euclid_div(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Euclid::euclid_div(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn euclid_rem(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::common::executive::Value::Object(
                                std::sync::Arc::new(melodium_core::executive::Euclid::euclid_rem(self, &obj))
                            )
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn euclid_div(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Euclid not implemented for {}", #name)
            }

            fn euclid_rem(&self, other: &melodium_core::common::executive::Value) -> melodium_core::common::executive::Value {
                panic!("Euclid not implemented for {}", #name)
            }
        }
    }
}

fn checked_euclid(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn checked_euclid_div(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::CheckedEuclid::checked_euclid_div(self, &obj)
                                .map(|obj| melodium_core::common::executive::Value::Object(
                                    std::sync::Arc::new(obj)
                                ))
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

            fn checked_euclid_rem(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                match other {
                    melodium_core::common::executive::Value::Object(obj) =>
                        if let Ok(obj) = std::sync::Arc::clone(obj).downcast_arc::<Self>() {
                            melodium_core::executive::CheckedEuclid::checked_euclid_rem(self, &obj)
                                .map(|obj| melodium_core::common::executive::Value::Object(
                                    std::sync::Arc::new(obj)
                                ))
                        }
                        else {
                            panic!("Invalid object type")
                        }
                    _ => panic!("Invalid value type")
                }
            }

        }
    } else {
        quote! {
            fn checked_euclid_div(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                panic!("CheckedEuclid not implemented for {}", #name)
            }

            fn checked_euclid_rem(&self, other: &melodium_core::common::executive::Value) -> Option<melodium_core::common::executive::Value> {
                panic!("CheckedEuclid not implemented for {}", #name)
            }
        }
    }
}
