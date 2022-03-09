
use crate::core::prelude::*;

macro_rules! impl_VectorToString {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $recv_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","vector";$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Vector,$mel_type,Stream)
            ],
            outputs![
                output!("value",Vector,String,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");
            
                'main: while let Ok(vecs_values) = input.$recv_func().await {
            
                    for vec_values in vecs_values {
                        ok_or_break!('main, output.send_vec_string(
                            vec_values.iter().map(|v| v.to_string()).collect()
                        ).await);
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_VectorToString!(u8_to_string, "U8ToString", U8, recv_vec_u8);
impl_VectorToString!(u16_to_string, "U16ToString", U16, recv_vec_u16);
impl_VectorToString!(u32_to_string, "U32ToString", U32, recv_vec_u32);
impl_VectorToString!(u64_to_string, "U64ToString", U64, recv_vec_u64);
impl_VectorToString!(u128_to_string, "U128ToString", U128, recv_vec_u128);
impl_VectorToString!(i8_to_string, "I8ToString", I8, recv_vec_i8);
impl_VectorToString!(i16_to_string, "I16ToString", I16, recv_vec_i16);
impl_VectorToString!(i32_to_string, "I32ToString", I32, recv_vec_i32);
impl_VectorToString!(i64_to_string, "I64ToString", I64, recv_vec_i64);
impl_VectorToString!(i128_to_string, "I128ToString", I128, recv_vec_i128);
impl_VectorToString!(f32_to_string, "F32ToString", F32, recv_vec_f32);
impl_VectorToString!(f64_to_string, "F64ToString", F64, recv_vec_f64);
impl_VectorToString!(bool_to_string, "BoolToString", Bool, recv_vec_bool);
impl_VectorToString!(byte_to_string, "ByteToString", Byte, recv_vec_byte);
impl_VectorToString!(char_to_string, "CharToString", Char, recv_vec_char);

pub fn register(mut c: &mut CollectionPool) {

    u8_to_string::register(&mut c);
    u16_to_string::register(&mut c);
    u32_to_string::register(&mut c);
    u64_to_string::register(&mut c);
    u128_to_string::register(&mut c);
    i8_to_string::register(&mut c);
    i16_to_string::register(&mut c);
    i32_to_string::register(&mut c);
    i64_to_string::register(&mut c);
    i128_to_string::register(&mut c);
    f32_to_string::register(&mut c);
    f64_to_string::register(&mut c);
    bool_to_string::register(&mut c);
    byte_to_string::register(&mut c);
    char_to_string::register(&mut c);

}

/*
    FOR DEVELOPERS

The lines can be regenerated as will using the following script:

```
#!/bin/bash

TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 bool byte char"

for TYPE in $TYPES
do
    UPPER_CASE_TYPE=${TYPE^}
    echo "impl_VectorToString!(${TYPE}_to_string, \"${UPPER_CASE_TYPE}ToString\", $UPPER_CASE_TYPE, recv_vec_$TYPE);"
    #echo "${TYPE}_to_string::register(&mut c);"

done
``` 
*/

