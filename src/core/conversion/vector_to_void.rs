
use crate::core::prelude::*;

macro_rules! impl_VectorToVoid {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $recv_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","vector";$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("vector",Vector,$mel_type,Stream)
            ],
            outputs![
                output!("pattern",Vector,Void,Stream)
            ],
            host {
                let input = host.get_input("vector");
                let output = host.get_output("pattern");
            
                while let Ok(vectors) = input.$recv_func().await {

                    let mut pattern = Vec::new();
                    for v in vectors {
                        pattern.push(vec![(); v.len()]);
                    }

                    ok_or_break!(output.send_multiple_vec_void(pattern).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_VectorToVoid!(u8_to_void, "U8ToVoid", U8, recv_vec_u8);
impl_VectorToVoid!(u16_to_void, "U16ToVoid", U16, recv_vec_u16);
impl_VectorToVoid!(u32_to_void, "U32ToVoid", U32, recv_vec_u32);
impl_VectorToVoid!(u64_to_void, "U64ToVoid", U64, recv_vec_u64);
impl_VectorToVoid!(u128_to_void, "U128ToVoid", U128, recv_vec_u128);
impl_VectorToVoid!(i8_to_void, "I8ToVoid", I8, recv_vec_i8);
impl_VectorToVoid!(i16_to_void, "I16ToVoid", I16, recv_vec_i16);
impl_VectorToVoid!(i32_to_void, "I32ToVoid", I32, recv_vec_i32);
impl_VectorToVoid!(i64_to_void, "I64ToVoid", I64, recv_vec_i64);
impl_VectorToVoid!(i128_to_void, "I128ToVoid", I128, recv_vec_i128);
impl_VectorToVoid!(f32_to_void, "F32ToVoid", F32, recv_vec_f32);
impl_VectorToVoid!(f64_to_void, "F64ToVoid", F64, recv_vec_f64);
impl_VectorToVoid!(bool_to_void, "BoolToVoid", Bool, recv_vec_bool);
impl_VectorToVoid!(byte_to_void, "ByteToVoid", Byte, recv_vec_byte);
impl_VectorToVoid!(char_to_void, "CharToVoid", Char, recv_vec_char);
impl_VectorToVoid!(string_to_void, "StringToVoid", String, recv_vec_string);

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
