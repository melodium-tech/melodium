
use crate::core::prelude::*;

macro_rules! impl_ScalarToString {
    ($mod:ident, $mel_name:expr, $type:ident, $mel_type:ident, $recv_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","scalar";$mel_name),
            formatdoc!(r"Convert stream of `{type}` into `string`.

            `{type}` gets converted into `string`, and the resulting strings are send through stream in continuity.", type = stringify!($type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("value",Scalar,String,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");
            
                'main: while let Ok(values) = input.$recv_func().await {
            
                    for value in values {
                        ok_or_break!('main, output.send_string(value.to_string()).await);
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_ScalarToString!(u8_to_string, "U8ToString", u8, U8, recv_u8);
impl_ScalarToString!(u16_to_string, "U16ToString", u16, U16, recv_u16);
impl_ScalarToString!(u32_to_string, "U32ToString", u32, U32, recv_u32);
impl_ScalarToString!(u64_to_string, "U64ToString", u64, U64, recv_u64);
impl_ScalarToString!(u128_to_string, "U128ToString", u128, U128, recv_u128);
impl_ScalarToString!(i8_to_string, "I8ToString", i8, I8, recv_i8);
impl_ScalarToString!(i16_to_string, "I16ToString", i16, I16, recv_i16);
impl_ScalarToString!(i32_to_string, "I32ToString", i32, I32, recv_i32);
impl_ScalarToString!(i64_to_string, "I64ToString", i64, I64, recv_i64);
impl_ScalarToString!(i128_to_string, "I128ToString", i128, I128, recv_i128);
impl_ScalarToString!(f32_to_string, "F32ToString", f32, F32, recv_f32);
impl_ScalarToString!(f64_to_string, "F64ToString", f64, F64, recv_f64);
impl_ScalarToString!(bool_to_string, "BoolToString", bool, Bool, recv_bool);
impl_ScalarToString!(byte_to_string, "ByteToString", byte, Byte, recv_byte);
impl_ScalarToString!(char_to_string, "CharToString", char, Char, recv_char);

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
    echo "impl_ScalarToString!(${TYPE}_to_string, \"${UPPER_CASE_TYPE}ToString\", $TYPE, $UPPER_CASE_TYPE, recv_$TYPE);"
    #echo "${TYPE}_to_string::register(&mut c);"

done
```
    
*/

