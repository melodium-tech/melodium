
use crate::core::prelude::*;

macro_rules! impl_ScalarToVoid {
    ($mod:ident, $mel_name:expr, $type:ident, $mel_type:ident, $recv_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","scalar";$mel_name),
            formatdoc!(r"Turns `{type}` stream into `void` one.

            Send one `iter` per input `value` received.", type = stringify!($type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("iter",Scalar,Void,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("iter");
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.send_multiple_void(vec![(); values.len()]).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_ScalarToVoid!(u8_to_void, "U8ToVoid", u8, U8, recv_u8);
impl_ScalarToVoid!(u16_to_void, "U16ToVoid", u16, U16, recv_u16);
impl_ScalarToVoid!(u32_to_void, "U32ToVoid", u32, U32, recv_u32);
impl_ScalarToVoid!(u64_to_void, "U64ToVoid", u64, U64, recv_u64);
impl_ScalarToVoid!(u128_to_void, "U128ToVoid", u128, U128, recv_u128);
impl_ScalarToVoid!(i8_to_void, "I8ToVoid", i8, I8, recv_i8);
impl_ScalarToVoid!(i16_to_void, "I16ToVoid", i16, I16, recv_i16);
impl_ScalarToVoid!(i32_to_void, "I32ToVoid", i32, I32, recv_i32);
impl_ScalarToVoid!(i64_to_void, "I64ToVoid", i64, I64, recv_i64);
impl_ScalarToVoid!(i128_to_void, "I128ToVoid", i128, I128, recv_i128);
impl_ScalarToVoid!(f32_to_void, "F32ToVoid", f32, F32, recv_f32);
impl_ScalarToVoid!(f64_to_void, "F64ToVoid", f64, F64, recv_f64);
impl_ScalarToVoid!(bool_to_void, "BoolToVoid", bool, Bool, recv_bool);
impl_ScalarToVoid!(byte_to_void, "ByteToVoid", byte, Byte, recv_byte);
impl_ScalarToVoid!(char_to_void, "CharToVoid", char, Char, recv_char);
impl_ScalarToVoid!(string_to_void, "StringToVoid", string, String, recv_string);

pub fn register(mut c: &mut CollectionPool) {

    u8_to_void::register(&mut c);
    u16_to_void::register(&mut c);
    u32_to_void::register(&mut c);
    u64_to_void::register(&mut c);
    u128_to_void::register(&mut c);
    i8_to_void::register(&mut c);
    i16_to_void::register(&mut c);
    i32_to_void::register(&mut c);
    i64_to_void::register(&mut c);
    i128_to_void::register(&mut c);
    f32_to_void::register(&mut c);
    f64_to_void::register(&mut c);
    bool_to_void::register(&mut c);
    byte_to_void::register(&mut c);
    char_to_void::register(&mut c);
    string_to_void::register(&mut c);
}
