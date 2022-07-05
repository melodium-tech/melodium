
use crate::core::prelude::*;

macro_rules! impl_VectorFloatToInteger {
    ($mod:ident, $mel_name:expr, $input_mel_type:ident, $recv_func:ident, $output_mel_value_type:ident, $output_mel_type:ident, $output_rust_type:ty, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","vector";$mel_name),
            models![],
            treatment_sources![],
            parameters![
                parameter!("neg_infinity",Var,Scalar,$output_mel_type,Some(Value::$output_mel_type(<$output_rust_type>::MIN))),
                parameter!("pos_infinity",Var,Scalar,$output_mel_type,Some(Value::$output_mel_type(<$output_rust_type>::MAX))),
                parameter!("nan",Var,Scalar,$output_mel_type,Some(Value::$output_mel_type(<$output_rust_type>::default())))
            ],
            inputs![
                input!("value",Vector,$input_mel_type,Stream)
            ],
            outputs![
                output!("value",Vector,$output_mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");

                let pos_infinity = host.get_parameter("pos_infinity").$output_mel_value_type();
                let neg_infinity = host.get_parameter("neg_infinity").$output_mel_value_type();
                let nan = host.get_parameter("nan").$output_mel_value_type();
            
                'main: while let Ok(vecs_numbers) = input.$recv_func().await {
            
                    for vec_number in vecs_numbers {

                        let output_vector: Vec<$output_rust_type> = vec_number.iter().map(|number| {
                            if number.is_finite() { *number as $output_rust_type }
                            else if number.is_nan() { nan }
                            else if number.is_sign_positive() { pos_infinity }
                            else /*if number.is_sign_negative()*/ { neg_infinity }
                        }).collect();

                        ok_or_break!('main, output.$send_func(output_vector).await);
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

// Conversions for f32
impl_VectorFloatToInteger!(f32_to_u8, "VectorF32ToU8", F32, recv_vec_f32, u8, U8, u8, send_vec_u8);
impl_VectorFloatToInteger!(f32_to_u16, "VectorF32ToU16", F32, recv_vec_f32, u16, U16, u16, send_vec_u16);
impl_VectorFloatToInteger!(f32_to_u32, "VectorF32ToU32", F32, recv_vec_f32, u32, U32, u32, send_vec_u32);
impl_VectorFloatToInteger!(f32_to_u64, "VectorF32ToU64", F32, recv_vec_f32, u64, U64, u64, send_vec_u64);
impl_VectorFloatToInteger!(f32_to_u128, "VectorF32ToU128", F32, recv_vec_f32, u128, U128, u128, send_vec_u128);
impl_VectorFloatToInteger!(f32_to_i8, "VectorF32ToI8", F32, recv_vec_f32, i8, I8, i8, send_vec_i8);
impl_VectorFloatToInteger!(f32_to_i16, "VectorF32ToI16", F32, recv_vec_f32, i16, I16, i16, send_vec_i16);
impl_VectorFloatToInteger!(f32_to_i32, "VectorF32ToI32", F32, recv_vec_f32, i32, I32, i32, send_vec_i32);
impl_VectorFloatToInteger!(f32_to_i64, "VectorF32ToI64", F32, recv_vec_f32, i64, I64, i64, send_vec_i64);
impl_VectorFloatToInteger!(f32_to_i128, "VectorF32ToI128", F32, recv_vec_f32, i128, I128, i128, send_vec_i128);

// Conversions for f64
impl_VectorFloatToInteger!(f64_to_u8, "VectorF64ToU8", F64, recv_vec_f64, u8, U8, u8, send_vec_u8);
impl_VectorFloatToInteger!(f64_to_u16, "VectorF64ToU16", F64, recv_vec_f64, u16, U16, u16, send_vec_u16);
impl_VectorFloatToInteger!(f64_to_u32, "VectorF64ToU32", F64, recv_vec_f64, u32, U32, u32, send_vec_u32);
impl_VectorFloatToInteger!(f64_to_u64, "VectorF64ToU64", F64, recv_vec_f64, u64, U64, u64, send_vec_u64);
impl_VectorFloatToInteger!(f64_to_u128, "VectorF64ToU128", F64, recv_vec_f64, u128, U128, u128, send_vec_u128);
impl_VectorFloatToInteger!(f64_to_i8, "VectorF64ToI8", F64, recv_vec_f64, i8, I8, i8, send_vec_i8);
impl_VectorFloatToInteger!(f64_to_i16, "VectorF64ToI16", F64, recv_vec_f64, i16, I16, i16, send_vec_i16);
impl_VectorFloatToInteger!(f64_to_i32, "VectorF64ToI32", F64, recv_vec_f64, i32, I32, i32, send_vec_i32);
impl_VectorFloatToInteger!(f64_to_i64, "VectorF64ToI64", F64, recv_vec_f64, i64, I64, i64, send_vec_i64);
impl_VectorFloatToInteger!(f64_to_i128, "VectorF64ToI128", F64, recv_vec_f64, i128, I128, i128, send_vec_i128);

pub fn register(mut c: &mut CollectionPool) {

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
           
           echo "impl_VectorFloatToInteger!(${FLOAT}_to_${INT}, \"Vector${UC_FLOAT}To${UC_INT}\", $UC_FLOAT, recv_vec_$FLOAT, $INT, $UC_INT, $INT, send_vec_$INT);"
           #echo "${FLOAT}_to_${INT}::register(&mut c);"
    done
    
    echo
done
```
    
*/

