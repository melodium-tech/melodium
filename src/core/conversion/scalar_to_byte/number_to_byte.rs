
use crate::core::prelude::*;

macro_rules! impl_ScalarToByte {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $recv_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","scalar";$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("data",Scalar,Byte,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("data");
            
                while let Ok(numbers) = input.$recv_func().await {
            
                    for number in numbers {
                        output.send_multiple_byte(number.to_be_bytes().to_vec()).await;
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_ScalarToByte!(u8_to_byte, "U8ToByte", U8, recv_u8);
impl_ScalarToByte!(u16_to_byte, "U16ToByte", U16, recv_u16);
impl_ScalarToByte!(u32_to_byte, "U32ToByte", U32, recv_u32);
impl_ScalarToByte!(u64_to_byte, "U64ToByte", U64, recv_u64);
impl_ScalarToByte!(u128_to_byte, "U128ToByte", U128, recv_u128);
impl_ScalarToByte!(i8_to_byte, "I8ToByte", I8, recv_i8);
impl_ScalarToByte!(i16_to_byte, "I16ToByte", I16, recv_i16);
impl_ScalarToByte!(i32_to_byte, "I32ToByte", I32, recv_i32);
impl_ScalarToByte!(i64_to_byte, "I64ToByte", I64, recv_i64);
impl_ScalarToByte!(i128_to_byte, "I128ToByte", I128, recv_i128);
impl_ScalarToByte!(f32_to_byte, "F32ToByte", F32, recv_f32);
impl_ScalarToByte!(f64_to_byte, "F64ToByte", F64, recv_f64);

pub fn register(c: &mut CollectionPool) {

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
    echo "impl_ScalarToByte!(${TYPE}_to_byte, \"${UPPER_CASE_TYPE}ToByte\", $UPPER_CASE_TYPE, recv_$TYPE);"
    #echo "${TYPE}_to_byte::register(&mut c);"

done
```
    
*/
