
use crate::core::prelude::*;

macro_rules! impl_ScalarFloatToInteger {
    ($mod:ident, $mel_name:expr, $input_mel_type:ident, $recv_func:ident, $output_mel_value_type:ident, $output_mel_type:ident, $output_rust_type:ty, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","scalar";$mel_name),
            models![],
            treatment_sources![],
            parameters![
                parameter!("neg_infinity",Scalar,$output_mel_type,Some(Value::$output_mel_type(<$output_rust_type>::MIN))),
                parameter!("pos_infinity",Scalar,$output_mel_type,Some(Value::$output_mel_type(<$output_rust_type>::MAX))),
                parameter!("nan",Scalar,$output_mel_type,Some(Value::$output_mel_type(<$output_rust_type>::default())))
            ],
            inputs![
                input!("value",Scalar,$input_mel_type,Stream)
            ],
            outputs![
                output!("value",Scalar,$output_mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");

                let pos_infinity = host.get_parameter("pos_infinity").$output_mel_value_type();
                let neg_infinity = host.get_parameter("neg_infinity").$output_mel_value_type();
                let nan = host.get_parameter("nan").$output_mel_value_type();
            
                while let Ok(numbers) = input.$recv_func().await {
            
                    for number in numbers {

                        let output_number =
                        if number.is_finite() { number as $output_rust_type }
                        else if number.is_nan() { nan }
                        else if number.is_sign_positive() { pos_infinity }
                        else /*if number.is_sign_negative()*/ { neg_infinity }
                        ;

                        output.$send_func(output_number).await;
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

// Conversions for f32
impl_ScalarFloatToInteger!(f32_to_u8, "ScalarF32ToU8", F32, recv_f32, u8, U8, u8, send_u8);
impl_ScalarFloatToInteger!(f32_to_u16, "ScalarF32ToU16", F32, recv_f32, u16, U16, u16, send_u16);
impl_ScalarFloatToInteger!(f32_to_u32, "ScalarF32ToU32", F32, recv_f32, u32, U32, u32, send_u32);
impl_ScalarFloatToInteger!(f32_to_u64, "ScalarF32ToU64", F32, recv_f32, u64, U64, u64, send_u64);
impl_ScalarFloatToInteger!(f32_to_u128, "ScalarF32ToU128", F32, recv_f32, u128, U128, u128, send_u128);
impl_ScalarFloatToInteger!(f32_to_i8, "ScalarF32ToI8", F32, recv_f32, i8, I8, i8, send_i8);
impl_ScalarFloatToInteger!(f32_to_i16, "ScalarF32ToI16", F32, recv_f32, i16, I16, i16, send_i16);
impl_ScalarFloatToInteger!(f32_to_i32, "ScalarF32ToI32", F32, recv_f32, i32, I32, i32, send_i32);
impl_ScalarFloatToInteger!(f32_to_i64, "ScalarF32ToI64", F32, recv_f32, i64, I64, i64, send_i64);
impl_ScalarFloatToInteger!(f32_to_i128, "ScalarF32ToI128", F32, recv_f32, i128, I128, i128, send_i128);

// Conversions for f64
impl_ScalarFloatToInteger!(f64_to_u8, "ScalarF64ToU8", F64, recv_f64, u8, U8, u8, send_u8);
impl_ScalarFloatToInteger!(f64_to_u16, "ScalarF64ToU16", F64, recv_f64, u16, U16, u16, send_u16);
impl_ScalarFloatToInteger!(f64_to_u32, "ScalarF64ToU32", F64, recv_f64, u32, U32, u32, send_u32);
impl_ScalarFloatToInteger!(f64_to_u64, "ScalarF64ToU64", F64, recv_f64, u64, U64, u64, send_u64);
impl_ScalarFloatToInteger!(f64_to_u128, "ScalarF64ToU128", F64, recv_f64, u128, U128, u128, send_u128);
impl_ScalarFloatToInteger!(f64_to_i8, "ScalarF64ToI8", F64, recv_f64, i8, I8, i8, send_i8);
impl_ScalarFloatToInteger!(f64_to_i16, "ScalarF64ToI16", F64, recv_f64, i16, I16, i16, send_i16);
impl_ScalarFloatToInteger!(f64_to_i32, "ScalarF64ToI32", F64, recv_f64, i32, I32, i32, send_i32);
impl_ScalarFloatToInteger!(f64_to_i64, "ScalarF64ToI64", F64, recv_f64, i64, I64, i64, send_i64);
impl_ScalarFloatToInteger!(f64_to_i128, "ScalarF64ToI128", F64, recv_f64, i128, I128, i128, send_i128);

pub fn register(c: &mut CollectionPool) {

    // Conversions for f32
    f32_to_u8::register(&mut c);
    f32_to_u16::register(&mut c);
    f32_to_u32::register(&mut c);
    f32_to_u64::register(&mut c);
    f32_to_u128::register(&mut c);
    f32_to_i8::register(&mut c);
    f32_to_i16::register(&mut c);
    f32_to_i32::register(&mut c);
    f32_to_i64::register(&mut c);
    f32_to_i128::register(&mut c);

    // Conversions for f64
    f64_to_u8::register(&mut c);
    f64_to_u16::register(&mut c);
    f64_to_u32::register(&mut c);
    f64_to_u64::register(&mut c);
    f64_to_u128::register(&mut c);
    f64_to_i8::register(&mut c);
    f64_to_i16::register(&mut c);
    f64_to_i32::register(&mut c);
    f64_to_i64::register(&mut c);
    f64_to_i128::register(&mut c);

}

/*
    FOR DEVELOPERS

The lines can be regenerated as will using the following script:

```
#!/bin/bash

FLOAT_TYPES="f32 f64"
INT_TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128"

for FLOAT in $FLOAT_TYPES
do
    UC_FLOAT=${FLOAT^}
    
    echo "// Conversions for $FLOAT"
    
    for INT in $INT_TYPES
    do
           UC_INT=${INT^}
           
           echo "impl_ScalarFloatToInteger!(${FLOAT}_to_${INT}, \"Scalar${UC_FLOAT}To${UC_INT}\", $UC_FLOAT, recv_$FLOAT, $INT, $UC_INT, $INT, send_$INT);"
           #echo "${FLOAT}_to_${INT}::register(&mut c);"
    done
    
    echo
done
    
```
    
*/
