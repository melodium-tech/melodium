
use crate::core::prelude::*;

macro_rules! impl_Linearize {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("linearize";$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("vector",Vector,$mel_type,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type,Stream)
            ],
            host {
                let input = host.get_input("vector");
                let output = host.get_output("value");
            
                'main: while let Ok(vectors) = input.$recv_func().await {

                    for vector in vectors {
                        ok_or_break!('main, output.$send_func(vector).await);
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_Linearize!(linearize_u8, "LinearizeU8", U8, recv_vec_u8, send_multiple_u8);
impl_Linearize!(linearize_u16, "LinearizeU16", U16, recv_vec_u16, send_multiple_u16);
impl_Linearize!(linearize_u32, "LinearizeU32", U32, recv_vec_u32, send_multiple_u32);
impl_Linearize!(linearize_u64, "LinearizeU64", U64, recv_vec_u64, send_multiple_u64);
impl_Linearize!(linearize_u128, "LinearizeU128", U128, recv_vec_u128, send_multiple_u128);
impl_Linearize!(linearize_i8, "LinearizeI8", I8, recv_vec_i8, send_multiple_i8);
impl_Linearize!(linearize_i16, "LinearizeI16", I16, recv_vec_i16, send_multiple_i16);
impl_Linearize!(linearize_i32, "LinearizeI32", I32, recv_vec_i32, send_multiple_i32);
impl_Linearize!(linearize_i64, "LinearizeI64", I64, recv_vec_i64, send_multiple_i64);
impl_Linearize!(linearize_i128, "LinearizeI128", I128, recv_vec_i128, send_multiple_i128);
impl_Linearize!(linearize_f32, "LinearizeF32", F32, recv_vec_f32, send_multiple_f32);
impl_Linearize!(linearize_f64, "LinearizeF64", F64, recv_vec_f64, send_multiple_f64);
impl_Linearize!(linearize_bool, "LinearizeBool", Bool, recv_vec_bool, send_multiple_bool);
impl_Linearize!(linearize_byte, "LinearizeByte", Byte, recv_vec_byte, send_multiple_byte);
impl_Linearize!(linearize_char, "LinearizeChar", Char, recv_vec_char, send_multiple_char);
impl_Linearize!(linearize_string, "LinearizeString", String, recv_vec_string, send_multiple_string);

pub fn register(mut c: &mut CollectionPool) {

    linearize_u8::register(&mut c);
    linearize_u16::register(&mut c);
    linearize_u32::register(&mut c);
    linearize_u64::register(&mut c);
    linearize_u128::register(&mut c);
    linearize_i8::register(&mut c);
    linearize_i16::register(&mut c);
    linearize_i32::register(&mut c);
    linearize_i64::register(&mut c);
    linearize_i128::register(&mut c);
    linearize_f32::register(&mut c);
    linearize_f64::register(&mut c);
    linearize_bool::register(&mut c);
    linearize_byte::register(&mut c);
    linearize_char::register(&mut c);
    linearize_string::register(&mut c);
}
