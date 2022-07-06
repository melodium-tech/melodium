
use crate::core::prelude::*;

macro_rules! impl_Filling {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("filling","vector";$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream),
                input!("pattern",Vector,Void,Stream)
            ],
            outputs![
                output!("value",Vector,$mel_type,Stream)
            ],
            host {
                let value = host.get_input("value");
                let pattern = host.get_input("pattern");
                let output = host.get_output("value");
            
                while let (Ok(value), Ok(pattern)) = futures::join!(value.$recv_func(), pattern.recv_one_vec_void()) {
            
                    ok_or_break!(output.$send_func(vec![value; pattern.len()]).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_Filling!(vector_u8, "FillVecU8", U8, u8, recv_one_u8, send_vec_u8);
impl_Filling!(vector_u16, "FillVecU16", U16, u16, recv_one_u16, send_vec_u16);
impl_Filling!(vector_u32, "FillVecU32", U32, u32, recv_one_u32, send_vec_u32);
impl_Filling!(vector_u64, "FillVecU64", U64, u64, recv_one_u64, send_vec_u64);
impl_Filling!(vector_u128, "FillVecU128", U128, u128, recv_one_u128, send_vec_u128);
impl_Filling!(vector_i8, "FillVecI8", I8, i8, recv_one_i8, send_vec_i8);
impl_Filling!(vector_i16, "FillVecI16", I16, i16, recv_one_i16, send_vec_i16);
impl_Filling!(vector_i32, "FillVecI32", I32, i32, recv_one_i32, send_vec_i32);
impl_Filling!(vector_i64, "FillVecI64", I64, i64, recv_one_i64, send_vec_i64);
impl_Filling!(vector_i128, "FillVecI128", I128, i128, recv_one_i128, send_vec_i128);
impl_Filling!(vector_f32, "FillVecF32", F32, f32, recv_one_f32, send_vec_f32);
impl_Filling!(vector_f64, "FillVecF64", F64, f64, recv_one_f64, send_vec_f64);
impl_Filling!(vector_bool, "FillVecBool", Bool, bool, recv_one_bool, send_vec_bool);
impl_Filling!(vector_byte, "FillVecByte", Byte, byte, recv_one_byte, send_vec_byte);
impl_Filling!(vector_char, "FillVecChar", Char, char, recv_one_char, send_vec_char);
impl_Filling!(vector_string, "FillVecString", String, string, recv_one_string, send_vec_string);

pub fn register(mut c: &mut CollectionPool) {

    vector_u8::register(&mut c);
    vector_u16::register(&mut c);
    vector_u32::register(&mut c);
    vector_u64::register(&mut c);
    vector_u128::register(&mut c);
    vector_i8::register(&mut c);
    vector_i16::register(&mut c);
    vector_i32::register(&mut c);
    vector_i64::register(&mut c);
    vector_i128::register(&mut c);
    vector_f32::register(&mut c);
    vector_f64::register(&mut c);
    vector_bool::register(&mut c);
    vector_byte::register(&mut c);
    vector_char::register(&mut c);
    vector_string::register(&mut c);
}
