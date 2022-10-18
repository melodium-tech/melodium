
use crate::core::prelude::*;

macro_rules! impl_ScalarToByte {
    ($mod:ident, $mel_name:expr, $rust_type:ident, $mel_type:ident, $recv_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","scalar";$mel_name),
            formatdoc!(r"Convert stream of `{type}` into `Vec<byte>`.

            `{type}` gets converted into `Vec<byte>`, each vector contains the bytes of the former scalar `{type}` it represents.", type = stringify!($rust_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("data",Vector,Byte,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("data");
            
                'main: while let Ok(numbers) = input.$recv_func().await {
            
                    for number in numbers {
                        ok_or_break!('main, output.send_vec_byte(number.to_be_bytes().to_vec()).await);
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_ScalarToByte!(u8_to_byte, "U8ToByte", u8, U8, recv_u8);
impl_ScalarToByte!(u16_to_byte, "U16ToByte", u16, U16, recv_u16);
impl_ScalarToByte!(u32_to_byte, "U32ToByte", u32, U32, recv_u32);
impl_ScalarToByte!(u64_to_byte, "U64ToByte", u64, U64, recv_u64);
impl_ScalarToByte!(u128_to_byte, "U128ToByte", u128, U128, recv_u128);
impl_ScalarToByte!(i8_to_byte, "I8ToByte", i8, I8, recv_i8);
impl_ScalarToByte!(i16_to_byte, "I16ToByte", i16, I16, recv_i16);
impl_ScalarToByte!(i32_to_byte, "I32ToByte", i32, I32, recv_i32);
impl_ScalarToByte!(i64_to_byte, "I64ToByte", i64, I64, recv_i64);
impl_ScalarToByte!(i128_to_byte, "I128ToByte", i128, I128, recv_i128);
impl_ScalarToByte!(f32_to_byte, "F32ToByte", f32, F32, recv_f32);
impl_ScalarToByte!(f64_to_byte, "F64ToByte", f64, F64, recv_f64);

pub fn register(mut c: &mut CollectionPool) {

    u8_to_byte::register(&mut c);
    u16_to_byte::register(&mut c);
    u32_to_byte::register(&mut c);
    u64_to_byte::register(&mut c);
    u128_to_byte::register(&mut c);
    i8_to_byte::register(&mut c);
    i16_to_byte::register(&mut c);
    i32_to_byte::register(&mut c);
    i64_to_byte::register(&mut c);
    i128_to_byte::register(&mut c);
    f32_to_byte::register(&mut c);
    f64_to_byte::register(&mut c);
}

/*
    FOR DEVELOPERS

The lines above can be regenerated as will using the following script:

```
#!/bin/bash

TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64"

for TYPE in $TYPES
do
    UPPER_CASE_TYPE=${TYPE^}
    echo "impl_ScalarToByte!(${TYPE}_to_byte, \"${UPPER_CASE_TYPE}ToByte\", $TYPE, $UPPER_CASE_TYPE, recv_$TYPE);"
    #echo "${TYPE}_to_byte::register(&mut c);"

done
```
    
*/
