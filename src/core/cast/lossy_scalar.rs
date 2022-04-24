
use super::super::prelude::*;

macro_rules! impl_CastScalar {
    ($mod:ident, $mel_name:expr, $input_mel_type:ident, $recv_func:ident, $output_mel_value_type:ident, $output_mel_type:ident, $output_rust_type:ty, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("cast","scalar";$mel_name),
            models![],
            treatment_sources![],
            parameters![
                parameter!("truncate", Var, Scalar, Bool, Some(Value::Bool(true))),
                parameter!("or_default", Var, Scalar, $output_mel_type, Some(Value::$output_mel_type(<$output_rust_type>::default())))
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

                if host.get_parameter("truncate").bool() {

                    while let Ok(numbers) = input.$recv_func().await {

                        ok_or_break!(output.$send_func(
                            numbers.iter().map(|v| *v as $output_rust_type).collect()
                        ).await);
            
                    }
                
                    ResultStatus::Ok
                }
                else {

                    use std::convert::TryFrom;

                    let default = host.get_parameter("or_default").$output_mel_value_type();

                    while let Ok(numbers) = input.$recv_func().await {

                        ok_or_break!(
                            output.$send_func(
                                numbers.iter().map(
                                    |v| {
                                        if let Ok(casted_data) = <$output_rust_type>::try_from(*v) {
                                            casted_data
                                        }
                                        else {
                                            default
                                        }
                                    }
                                ).collect()
                            ).await
                        );
            
                    }
                
                    ResultStatus::Ok
                }
            
                
            }
        );
    }
}

// Lossy casts for u8
impl_CastScalar!(u8_to_i8, "CastScalarU8ToI8", U8, recv_u8, i8, I8, i8, send_multiple_i8);

// Lossy casts for u16
impl_CastScalar!(u16_to_u8, "CastScalarU16ToU8", U16, recv_u16, u8, U8, u8, send_multiple_u8);
impl_CastScalar!(u16_to_i8, "CastScalarU16ToI8", U16, recv_u16, i8, I8, i8, send_multiple_i8);
impl_CastScalar!(u16_to_i16, "CastScalarU16ToI16", U16, recv_u16, i16, I16, i16, send_multiple_i16);

// Lossy casts for u32
impl_CastScalar!(u32_to_u8, "CastScalarU32ToU8", U32, recv_u32, u8, U8, u8, send_multiple_u8);
impl_CastScalar!(u32_to_u16, "CastScalarU32ToU16", U32, recv_u32, u16, U16, u16, send_multiple_u16);
impl_CastScalar!(u32_to_i8, "CastScalarU32ToI8", U32, recv_u32, i8, I8, i8, send_multiple_i8);
impl_CastScalar!(u32_to_i16, "CastScalarU32ToI16", U32, recv_u32, i16, I16, i16, send_multiple_i16);
impl_CastScalar!(u32_to_i32, "CastScalarU32ToI32", U32, recv_u32, i32, I32, i32, send_multiple_i32);

// Lossy casts for u64
impl_CastScalar!(u64_to_u8, "CastScalarU64ToU8", U64, recv_u64, u8, U8, u8, send_multiple_u8);
impl_CastScalar!(u64_to_u16, "CastScalarU64ToU16", U64, recv_u64, u16, U16, u16, send_multiple_u16);
impl_CastScalar!(u64_to_u32, "CastScalarU64ToU32", U64, recv_u64, u32, U32, u32, send_multiple_u32);
impl_CastScalar!(u64_to_i8, "CastScalarU64ToI8", U64, recv_u64, i8, I8, i8, send_multiple_i8);
impl_CastScalar!(u64_to_i16, "CastScalarU64ToI16", U64, recv_u64, i16, I16, i16, send_multiple_i16);
impl_CastScalar!(u64_to_i32, "CastScalarU64ToI32", U64, recv_u64, i32, I32, i32, send_multiple_i32);
impl_CastScalar!(u64_to_i64, "CastScalarU64ToI64", U64, recv_u64, i64, I64, i64, send_multiple_i64);

// Lossy casts for u128
impl_CastScalar!(u128_to_u8, "CastScalarU128ToU8", U128, recv_u128, u8, U8, u8, send_multiple_u8);
impl_CastScalar!(u128_to_u16, "CastScalarU128ToU16", U128, recv_u128, u16, U16, u16, send_multiple_u16);
impl_CastScalar!(u128_to_u32, "CastScalarU128ToU32", U128, recv_u128, u32, U32, u32, send_multiple_u32);
impl_CastScalar!(u128_to_u64, "CastScalarU128ToU64", U128, recv_u128, u64, U64, u64, send_multiple_u64);
impl_CastScalar!(u128_to_i8, "CastScalarU128ToI8", U128, recv_u128, i8, I8, i8, send_multiple_i8);
impl_CastScalar!(u128_to_i16, "CastScalarU128ToI16", U128, recv_u128, i16, I16, i16, send_multiple_i16);
impl_CastScalar!(u128_to_i32, "CastScalarU128ToI32", U128, recv_u128, i32, I32, i32, send_multiple_i32);
impl_CastScalar!(u128_to_i64, "CastScalarU128ToI64", U128, recv_u128, i64, I64, i64, send_multiple_i64);
impl_CastScalar!(u128_to_i128, "CastScalarU128ToI128", U128, recv_u128, i128, I128, i128, send_multiple_i128);

// Lossy casts for i8
impl_CastScalar!(i8_to_u8, "CastScalarI8ToU8", I8, recv_i8, u8, U8, u8, send_multiple_u8);
impl_CastScalar!(i8_to_u16, "CastScalarI8ToU16", I8, recv_i8, u16, U16, u16, send_multiple_u16);
impl_CastScalar!(i8_to_u32, "CastScalarI8ToU32", I8, recv_i8, u32, U32, u32, send_multiple_u32);
impl_CastScalar!(i8_to_u64, "CastScalarI8ToU64", I8, recv_i8, u64, U64, u64, send_multiple_u64);
impl_CastScalar!(i8_to_u128, "CastScalarI8ToU128", I8, recv_i8, u128, U128, u128, send_multiple_u128);

// Lossy casts for i16
impl_CastScalar!(i16_to_u8, "CastScalarI16ToU8", I16, recv_i16, u8, U8, u8, send_multiple_u8);
impl_CastScalar!(i16_to_u16, "CastScalarI16ToU16", I16, recv_i16, u16, U16, u16, send_multiple_u16);
impl_CastScalar!(i16_to_u32, "CastScalarI16ToU32", I16, recv_i16, u32, U32, u32, send_multiple_u32);
impl_CastScalar!(i16_to_u64, "CastScalarI16ToU64", I16, recv_i16, u64, U64, u64, send_multiple_u64);
impl_CastScalar!(i16_to_u128, "CastScalarI16ToU128", I16, recv_i16, u128, U128, u128, send_multiple_u128);
impl_CastScalar!(i16_to_i8, "CastScalarI16ToI8", I16, recv_i16, i8, I8, i8, send_multiple_i8);

// Lossy casts for i32
impl_CastScalar!(i32_to_u8, "CastScalarI32ToU8", I32, recv_i32, u8, U8, u8, send_multiple_u8);
impl_CastScalar!(i32_to_u16, "CastScalarI32ToU16", I32, recv_i32, u16, U16, u16, send_multiple_u16);
impl_CastScalar!(i32_to_u32, "CastScalarI32ToU32", I32, recv_i32, u32, U32, u32, send_multiple_u32);
impl_CastScalar!(i32_to_u64, "CastScalarI32ToU64", I32, recv_i32, u64, U64, u64, send_multiple_u64);
impl_CastScalar!(i32_to_u128, "CastScalarI32ToU128", I32, recv_i32, u128, U128, u128, send_multiple_u128);
impl_CastScalar!(i32_to_i8, "CastScalarI32ToI8", I32, recv_i32, i8, I8, i8, send_multiple_i8);
impl_CastScalar!(i32_to_i16, "CastScalarI32ToI16", I32, recv_i32, i16, I16, i16, send_multiple_i16);

// Lossy casts for i64
impl_CastScalar!(i64_to_u8, "CastScalarI64ToU8", I64, recv_i64, u8, U8, u8, send_multiple_u8);
impl_CastScalar!(i64_to_u16, "CastScalarI64ToU16", I64, recv_i64, u16, U16, u16, send_multiple_u16);
impl_CastScalar!(i64_to_u32, "CastScalarI64ToU32", I64, recv_i64, u32, U32, u32, send_multiple_u32);
impl_CastScalar!(i64_to_u64, "CastScalarI64ToU64", I64, recv_i64, u64, U64, u64, send_multiple_u64);
impl_CastScalar!(i64_to_u128, "CastScalarI64ToU128", I64, recv_i64, u128, U128, u128, send_multiple_u128);
impl_CastScalar!(i64_to_i8, "CastScalarI64ToI8", I64, recv_i64, i8, I8, i8, send_multiple_i8);
impl_CastScalar!(i64_to_i16, "CastScalarI64ToI16", I64, recv_i64, i16, I16, i16, send_multiple_i16);
impl_CastScalar!(i64_to_i32, "CastScalarI64ToI32", I64, recv_i64, i32, I32, i32, send_multiple_i32);

// Lossy casts for i128
impl_CastScalar!(i128_to_u8, "CastScalarI128ToU8", I128, recv_i128, u8, U8, u8, send_multiple_u8);
impl_CastScalar!(i128_to_u16, "CastScalarI128ToU16", I128, recv_i128, u16, U16, u16, send_multiple_u16);
impl_CastScalar!(i128_to_u32, "CastScalarI128ToU32", I128, recv_i128, u32, U32, u32, send_multiple_u32);
impl_CastScalar!(i128_to_u64, "CastScalarI128ToU64", I128, recv_i128, u64, U64, u64, send_multiple_u64);
impl_CastScalar!(i128_to_u128, "CastScalarI128ToU128", I128, recv_i128, u128, U128, u128, send_multiple_u128);
impl_CastScalar!(i128_to_i8, "CastScalarI128ToI8", I128, recv_i128, i8, I8, i8, send_multiple_i8);
impl_CastScalar!(i128_to_i16, "CastScalarI128ToI16", I128, recv_i128, i16, I16, i16, send_multiple_i16);
impl_CastScalar!(i128_to_i32, "CastScalarI128ToI32", I128, recv_i128, i32, I32, i32, send_multiple_i32);
impl_CastScalar!(i128_to_i64, "CastScalarI128ToI64", I128, recv_i128, i64, I64, i64, send_multiple_i64);

pub fn register(mut c: &mut CollectionPool) {

    // Lossy casts for u8
    u8_to_i8::register(&mut c);

    // Lossy casts for u16
    u16_to_u8::register(&mut c);
    u16_to_i8::register(&mut c);
    u16_to_i16::register(&mut c);

    // Lossy casts for u32
    u32_to_u8::register(&mut c);
    u32_to_u16::register(&mut c);
    u32_to_i8::register(&mut c);
    u32_to_i16::register(&mut c);
    u32_to_i32::register(&mut c);

    // Lossy casts for u64
    u64_to_u8::register(&mut c);
    u64_to_u16::register(&mut c);
    u64_to_u32::register(&mut c);
    u64_to_i8::register(&mut c);
    u64_to_i16::register(&mut c);
    u64_to_i32::register(&mut c);
    u64_to_i64::register(&mut c);

    // Lossy casts for u128
    u128_to_u8::register(&mut c);
    u128_to_u16::register(&mut c);
    u128_to_u32::register(&mut c);
    u128_to_u64::register(&mut c);
    u128_to_i8::register(&mut c);
    u128_to_i16::register(&mut c);
    u128_to_i32::register(&mut c);
    u128_to_i64::register(&mut c);
    u128_to_i128::register(&mut c);

    // Lossy casts for i8
    i8_to_u8::register(&mut c);
    i8_to_u16::register(&mut c);
    i8_to_u32::register(&mut c);
    i8_to_u64::register(&mut c);
    i8_to_u128::register(&mut c);

    // Lossy casts for i16
    i16_to_u8::register(&mut c);
    i16_to_u16::register(&mut c);
    i16_to_u32::register(&mut c);
    i16_to_u64::register(&mut c);
    i16_to_u128::register(&mut c);
    i16_to_i8::register(&mut c);

    // Lossy casts for i32
    i32_to_u8::register(&mut c);
    i32_to_u16::register(&mut c);
    i32_to_u32::register(&mut c);
    i32_to_u64::register(&mut c);
    i32_to_u128::register(&mut c);
    i32_to_i8::register(&mut c);
    i32_to_i16::register(&mut c);

    // Lossy casts for i64
    i64_to_u8::register(&mut c);
    i64_to_u16::register(&mut c);
    i64_to_u32::register(&mut c);
    i64_to_u64::register(&mut c);
    i64_to_u128::register(&mut c);
    i64_to_i8::register(&mut c);
    i64_to_i16::register(&mut c);
    i64_to_i32::register(&mut c);

    // Lossy casts for i128
    i128_to_u8::register(&mut c);
    i128_to_u16::register(&mut c);
    i128_to_u32::register(&mut c);
    i128_to_u64::register(&mut c);
    i128_to_u128::register(&mut c);
    i128_to_i8::register(&mut c);
    i128_to_i16::register(&mut c);
    i128_to_i32::register(&mut c);
    i128_to_i64::register(&mut c);

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
    
    DISQUALIFIED_TYPES=`echo $TYPES | sed s/$TYPE//g`
    for QUALIFIED_TYPE in $QUALIFIED_TYPES
    do
        DISQUALIFIED_TYPES=`echo $DISQUALIFIED_TYPES | sed s/$QUALIFIED_TYPE//g`
    done
    
    echo "// Lossy casts for $TYPE"
    
    UPPER_CASE_TYPE=`echo $TYPE | tr '[:lower:]' '[:upper:]'`
    for CAST_TYPE in $DISQUALIFIED_TYPES
    do
        UPPER_CASE_CAST_TYPE=`echo $CAST_TYPE | tr '[:lower:]' '[:upper:]'`
        
        echo "impl_CastScalar!(${TYPE}_to_${CAST_TYPE}, \"CastScalar${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}\", $UPPER_CASE_TYPE, recv_$TYPE, $CAST_TYPE, $UPPER_CASE_CAST_TYPE, $CAST_TYPE, send_multiple_$CAST_TYPE);"
        #echo "${TYPE}_to_${CAST_TYPE}::register(&mut c);"
    done
    
    echo 
done
```
    
*/