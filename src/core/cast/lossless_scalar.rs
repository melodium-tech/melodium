
use crate::core::prelude::*;

macro_rules! impl_CastScalar {
    ($mod:ident, $mel_name:expr, $input_mel_type:ident, $recv_func:ident, $output_mel_type:ident, $output_rust_type:ty, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("cast","scalar";$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$input_mel_type,Stream)
            ],
            outputs![
                output!("value",Scalar,$output_mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");
            
                while let Ok(numbers) = input.$recv_func().await {
            
                    for number in numbers {
                        output.$send_func(number as $output_rust_type).await;
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

// Casts for u8
impl_CastScalar!(u8_to_u16, "U8ToU16", U8, recv_u8, U16, u16, send_u16);
impl_CastScalar!(u8_to_u32, "U8ToU32", U8, recv_u8, U32, u32, send_u32);
impl_CastScalar!(u8_to_u64, "U8ToU64", U8, recv_u8, U64, u64, send_u64);
impl_CastScalar!(u8_to_u128, "U8ToU128", U8, recv_u8, U128, u128, send_u128);
impl_CastScalar!(u8_to_i16, "U8ToI16", U8, recv_u8, I16, i16, send_i16);
impl_CastScalar!(u8_to_i32, "U8ToI32", U8, recv_u8, I32, i32, send_i32);
impl_CastScalar!(u8_to_i64, "U8ToI64", U8, recv_u8, I64, i64, send_i64);
impl_CastScalar!(u8_to_i128, "U8ToI128", U8, recv_u8, I128, i128, send_i128);
impl_CastScalar!(u8_to_f32, "U8ToF32", U8, recv_u8, F32, f32, send_f32);
impl_CastScalar!(u8_to_f64, "U8ToF64", U8, recv_u8, F64, f64, send_f64);

// Casts for u16
impl_CastScalar!(u16_to_u32, "U16ToU32", U16, recv_u16, U32, u32, send_u32);
impl_CastScalar!(u16_to_u64, "U16ToU64", U16, recv_u16, U64, u64, send_u64);
impl_CastScalar!(u16_to_u128, "U16ToU128", U16, recv_u16, U128, u128, send_u128);
impl_CastScalar!(u16_to_i32, "U16ToI32", U16, recv_u16, I32, i32, send_i32);
impl_CastScalar!(u16_to_i64, "U16ToI64", U16, recv_u16, I64, i64, send_i64);
impl_CastScalar!(u16_to_i128, "U16ToI128", U16, recv_u16, I128, i128, send_i128);
impl_CastScalar!(u16_to_f32, "U16ToF32", U16, recv_u16, F32, f32, send_f32);
impl_CastScalar!(u16_to_f64, "U16ToF64", U16, recv_u16, F64, f64, send_f64);

// Casts for u32
impl_CastScalar!(u32_to_u64, "U32ToU64", U32, recv_u32, U64, u64, send_u64);
impl_CastScalar!(u32_to_u128, "U32ToU128", U32, recv_u32, U128, u128, send_u128);
impl_CastScalar!(u32_to_i64, "U32ToI64", U32, recv_u32, I64, i64, send_i64);
impl_CastScalar!(u32_to_i128, "U32ToI128", U32, recv_u32, I128, i128, send_i128);
impl_CastScalar!(u32_to_f32, "U32ToF32", U32, recv_u32, F32, f32, send_f32);
impl_CastScalar!(u32_to_f64, "U32ToF64", U32, recv_u32, F64, f64, send_f64);

// Casts for u64
impl_CastScalar!(u64_to_u128, "U64ToU128", U64, recv_u64, U128, u128, send_u128);
impl_CastScalar!(u64_to_i128, "U64ToI128", U64, recv_u64, I128, i128, send_i128);
impl_CastScalar!(u64_to_f32, "U64ToF32", U64, recv_u64, F32, f32, send_f32);
impl_CastScalar!(u64_to_f64, "U64ToF64", U64, recv_u64, F64, f64, send_f64);

// Casts for u128
impl_CastScalar!(u128_to_f32, "U128ToF32", U128, recv_u128, F32, f32, send_f32);
impl_CastScalar!(u128_to_f64, "U128ToF64", U128, recv_u128, F64, f64, send_f64);

// Casts for i8
impl_CastScalar!(i8_to_i16, "I8ToI16", I8, recv_i8, I16, i16, send_i16);
impl_CastScalar!(i8_to_i32, "I8ToI32", I8, recv_i8, I32, i32, send_i32);
impl_CastScalar!(i8_to_i64, "I8ToI64", I8, recv_i8, I64, i64, send_i64);
impl_CastScalar!(i8_to_i128, "I8ToI128", I8, recv_i8, I128, i128, send_i128);
impl_CastScalar!(i8_to_f32, "I8ToF32", I8, recv_i8, F32, f32, send_f32);
impl_CastScalar!(i8_to_f64, "I8ToF64", I8, recv_i8, F64, f64, send_f64);

// Casts for i16
impl_CastScalar!(i16_to_i32, "I16ToI32", I16, recv_i16, I32, i32, send_i32);
impl_CastScalar!(i16_to_i64, "I16ToI64", I16, recv_i16, I64, i64, send_i64);
impl_CastScalar!(i16_to_i128, "I16ToI128", I16, recv_i16, I128, i128, send_i128);
impl_CastScalar!(i16_to_f32, "I16ToF32", I16, recv_i16, F32, f32, send_f32);
impl_CastScalar!(i16_to_f64, "I16ToF64", I16, recv_i16, F64, f64, send_f64);

// Casts for i32
impl_CastScalar!(i32_to_i64, "I32ToI64", I32, recv_i32, I64, i64, send_i64);
impl_CastScalar!(i32_to_i128, "I32ToI128", I32, recv_i32, I128, i128, send_i128);
impl_CastScalar!(i32_to_f32, "I32ToF32", I32, recv_i32, F32, f32, send_f32);
impl_CastScalar!(i32_to_f64, "I32ToF64", I32, recv_i32, F64, f64, send_f64);

// Casts for i64
impl_CastScalar!(i64_to_i128, "I64ToI128", I64, recv_i64, I128, i128, send_i128);
impl_CastScalar!(i64_to_f32, "I64ToF32", I64, recv_i64, F32, f32, send_f32);
impl_CastScalar!(i64_to_f64, "I64ToF64", I64, recv_i64, F64, f64, send_f64);

// Casts for i128
impl_CastScalar!(i128_to_f32, "I128ToF32", I128, recv_i128, F32, f32, send_f32);
impl_CastScalar!(i128_to_f64, "I128ToF64", I128, recv_i128, F64, f64, send_f64);


pub fn register(mut c: &mut CollectionPool) {

    // Casts for u8
    u8_to_u16::register(&mut c);
    u8_to_u32::register(&mut c);
    u8_to_u64::register(&mut c);
    u8_to_u128::register(&mut c);
    u8_to_i16::register(&mut c);
    u8_to_i32::register(&mut c);
    u8_to_i64::register(&mut c);
    u8_to_i128::register(&mut c);
    u8_to_f32::register(&mut c);
    u8_to_f64::register(&mut c);

    // Casts for u16
    u16_to_u32::register(&mut c);
    u16_to_u64::register(&mut c);
    u16_to_u128::register(&mut c);
    u16_to_i32::register(&mut c);
    u16_to_i64::register(&mut c);
    u16_to_i128::register(&mut c);
    u16_to_f32::register(&mut c);
    u16_to_f64::register(&mut c);

    // Casts for u32
    u32_to_u64::register(&mut c);
    u32_to_u128::register(&mut c);
    u32_to_i64::register(&mut c);
    u32_to_i128::register(&mut c);
    u32_to_f32::register(&mut c);
    u32_to_f64::register(&mut c);

    // Casts for u64
    u64_to_u128::register(&mut c);
    u64_to_i128::register(&mut c);
    u64_to_f32::register(&mut c);
    u64_to_f64::register(&mut c);

    // Casts for u128
    u128_to_f32::register(&mut c);
    u128_to_f64::register(&mut c);

    // Casts for i8
    i8_to_i16::register(&mut c);
    i8_to_i32::register(&mut c);
    i8_to_i64::register(&mut c);
    i8_to_i128::register(&mut c);
    i8_to_f32::register(&mut c);
    i8_to_f64::register(&mut c);

    // Casts for i16
    i16_to_i32::register(&mut c);
    i16_to_i64::register(&mut c);
    i16_to_i128::register(&mut c);
    i16_to_f32::register(&mut c);
    i16_to_f64::register(&mut c);

    // Casts for i32
    i32_to_i64::register(&mut c);
    i32_to_i128::register(&mut c);
    i32_to_f32::register(&mut c);
    i32_to_f64::register(&mut c);

    // Casts for i64
    i64_to_i128::register(&mut c);
    i64_to_f32::register(&mut c);
    i64_to_f64::register(&mut c);

    // Casts for i128
    i128_to_f32::register(&mut c);
    i128_to_f64::register(&mut c);

}

/*
    FOR DEVELOPERS

The lines about u/i* casts can be regenerated as will using the following script:

```
#!/bin/bash

TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128"

for TYPE in $TYPES
do
    TYPE_SIG=`echo $TYPE | grep -o [a-z]`
    TYPE_SIZE=`echo $TYPE | grep -oE [0-9]+`
    
    QUALIFIED_TYPES="$TYPES"
    if [ $TYPE_SIG == 'i' ]
    then
        QUALIFIED_TYPES=`echo $QUALIFIED_TYPES | sed -E s/u[0-9]+//g`
    fi
    
    while [ $TYPE_SIZE -ge 8 ]
    do
        QUALIFIED_TYPES=`echo $QUALIFIED_TYPES | sed s/[a-z]$TYPE_SIZE//g`
        TYPE_SIZE=`expr $TYPE_SIZE / 2`
    done
    
    QUALIFIED_TYPES="$QUALIFIED_TYPES f32 f64"
    
    echo "// Casts for $TYPE"
    
    UPPER_CASE_TYPE=`echo $TYPE | tr '[:lower:]' '[:upper:]'`
    for CAST_TYPE in $QUALIFIED_TYPES
    do
        UPPER_CASE_CAST_TYPE=`echo $CAST_TYPE | tr '[:lower:]' '[:upper:]'`
        
        echo "impl_CastScalar!(${TYPE}_to_${CAST_TYPE}, \"${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}\", $UPPER_CASE_TYPE, recv_$TYPE, $UPPER_CASE_CAST_TYPE, $CAST_TYPE, send_$CAST_TYPE);"
        #echo "${TYPE}_to_${CAST_TYPE}::register(&mut c);"
    done
    
    echo 
done
```

*/