
use crate::core::prelude::*;

macro_rules! impl_CastVector {
    ($mod:ident, $mel_name:expr, $input_mel_type:ident, $recv_func:ident, $output_mel_type:ident, $output_rust_type:ty, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("cast","vector";$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Vector,$input_mel_type,Stream)
            ],
            outputs![
                output!("value",Vector,$output_mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");
            
                while let Ok(vecs_numbers) = input.$recv_func().await {
            
                    for vec_numbers in vecs_numbers {
                        output.$send_func(
                            vec_numbers.iter().map(|v| *v as $output_rust_type).collect()
                        ).await;
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

// Casts for u8
impl_CastVector!(u8_to_u16, "VecU8ToVecU16", U8, recv_vec_u8, U16, u16, send_vec_u16);
impl_CastVector!(u8_to_u32, "VecU8ToVecU32", U8, recv_vec_u8, U32, u32, send_vec_u32);
impl_CastVector!(u8_to_u64, "VecU8ToVecU64", U8, recv_vec_u8, U64, u64, send_vec_u64);
impl_CastVector!(u8_to_u128, "VecU8ToVecU128", U8, recv_vec_u8, U128, u128, send_vec_u128);
impl_CastVector!(u8_to_i16, "VecU8ToVecI16", U8, recv_vec_u8, I16, i16, send_vec_i16);
impl_CastVector!(u8_to_i32, "VecU8ToVecI32", U8, recv_vec_u8, I32, i32, send_vec_i32);
impl_CastVector!(u8_to_i64, "VecU8ToVecI64", U8, recv_vec_u8, I64, i64, send_vec_i64);
impl_CastVector!(u8_to_i128, "VecU8ToVecI128", U8, recv_vec_u8, I128, i128, send_vec_i128);
impl_CastVector!(u8_to_f32, "VecU8ToVecF32", U8, recv_vec_u8, F32, f32, send_vec_f32);
impl_CastVector!(u8_to_f64, "VecU8ToVecF64", U8, recv_vec_u8, F64, f64, send_vec_f64);

// Casts for u16
impl_CastVector!(u16_to_u32, "VecU16ToVecU32", U16, recv_vec_u16, U32, u32, send_vec_u32);
impl_CastVector!(u16_to_u64, "VecU16ToVecU64", U16, recv_vec_u16, U64, u64, send_vec_u64);
impl_CastVector!(u16_to_u128, "VecU16ToVecU128", U16, recv_vec_u16, U128, u128, send_vec_u128);
impl_CastVector!(u16_to_i32, "VecU16ToVecI32", U16, recv_vec_u16, I32, i32, send_vec_i32);
impl_CastVector!(u16_to_i64, "VecU16ToVecI64", U16, recv_vec_u16, I64, i64, send_vec_i64);
impl_CastVector!(u16_to_i128, "VecU16ToVecI128", U16, recv_vec_u16, I128, i128, send_vec_i128);
impl_CastVector!(u16_to_f32, "VecU16ToVecF32", U16, recv_vec_u16, F32, f32, send_vec_f32);
impl_CastVector!(u16_to_f64, "VecU16ToVecF64", U16, recv_vec_u16, F64, f64, send_vec_f64);

// Casts for u32
impl_CastVector!(u32_to_u64, "VecU32ToVecU64", U32, recv_vec_u32, U64, u64, send_vec_u64);
impl_CastVector!(u32_to_u128, "VecU32ToVecU128", U32, recv_vec_u32, U128, u128, send_vec_u128);
impl_CastVector!(u32_to_i64, "VecU32ToVecI64", U32, recv_vec_u32, I64, i64, send_vec_i64);
impl_CastVector!(u32_to_i128, "VecU32ToVecI128", U32, recv_vec_u32, I128, i128, send_vec_i128);
impl_CastVector!(u32_to_f32, "VecU32ToVecF32", U32, recv_vec_u32, F32, f32, send_vec_f32);
impl_CastVector!(u32_to_f64, "VecU32ToVecF64", U32, recv_vec_u32, F64, f64, send_vec_f64);

// Casts for u64
impl_CastVector!(u64_to_u128, "VecU64ToVecU128", U64, recv_vec_u64, U128, u128, send_vec_u128);
impl_CastVector!(u64_to_i128, "VecU64ToVecI128", U64, recv_vec_u64, I128, i128, send_vec_i128);
impl_CastVector!(u64_to_f32, "VecU64ToVecF32", U64, recv_vec_u64, F32, f32, send_vec_f32);
impl_CastVector!(u64_to_f64, "VecU64ToVecF64", U64, recv_vec_u64, F64, f64, send_vec_f64);

// Casts for u128
impl_CastVector!(u128_to_f32, "VecU128ToVecF32", U128, recv_vec_u128, F32, f32, send_vec_f32);
impl_CastVector!(u128_to_f64, "VecU128ToVecF64", U128, recv_vec_u128, F64, f64, send_vec_f64);

// Casts for i8
impl_CastVector!(i8_to_i16, "VecI8ToVecI16", I8, recv_vec_i8, I16, i16, send_vec_i16);
impl_CastVector!(i8_to_i32, "VecI8ToVecI32", I8, recv_vec_i8, I32, i32, send_vec_i32);
impl_CastVector!(i8_to_i64, "VecI8ToVecI64", I8, recv_vec_i8, I64, i64, send_vec_i64);
impl_CastVector!(i8_to_i128, "VecI8ToVecI128", I8, recv_vec_i8, I128, i128, send_vec_i128);
impl_CastVector!(i8_to_f32, "VecI8ToVecF32", I8, recv_vec_i8, F32, f32, send_vec_f32);
impl_CastVector!(i8_to_f64, "VecI8ToVecF64", I8, recv_vec_i8, F64, f64, send_vec_f64);

// Casts for i16
impl_CastVector!(i16_to_i32, "VecI16ToVecI32", I16, recv_vec_i16, I32, i32, send_vec_i32);
impl_CastVector!(i16_to_i64, "VecI16ToVecI64", I16, recv_vec_i16, I64, i64, send_vec_i64);
impl_CastVector!(i16_to_i128, "VecI16ToVecI128", I16, recv_vec_i16, I128, i128, send_vec_i128);
impl_CastVector!(i16_to_f32, "VecI16ToVecF32", I16, recv_vec_i16, F32, f32, send_vec_f32);
impl_CastVector!(i16_to_f64, "VecI16ToVecF64", I16, recv_vec_i16, F64, f64, send_vec_f64);

// Casts for i32
impl_CastVector!(i32_to_i64, "VecI32ToVecI64", I32, recv_vec_i32, I64, i64, send_vec_i64);
impl_CastVector!(i32_to_i128, "VecI32ToVecI128", I32, recv_vec_i32, I128, i128, send_vec_i128);
impl_CastVector!(i32_to_f32, "VecI32ToVecF32", I32, recv_vec_i32, F32, f32, send_vec_f32);
impl_CastVector!(i32_to_f64, "VecI32ToVecF64", I32, recv_vec_i32, F64, f64, send_vec_f64);

// Casts for i64
impl_CastVector!(i64_to_i128, "VecI64ToVecI128", I64, recv_vec_i64, I128, i128, send_vec_i128);
impl_CastVector!(i64_to_f32, "VecI64ToVecF32", I64, recv_vec_i64, F32, f32, send_vec_f32);
impl_CastVector!(i64_to_f64, "VecI64ToVecF64", I64, recv_vec_i64, F64, f64, send_vec_f64);

// Casts for i128
impl_CastVector!(i128_to_f32, "VecI128ToVecF32", I128, recv_vec_i128, F32, f32, send_vec_f32);
impl_CastVector!(i128_to_f64, "VecI128ToVecF64", I128, recv_vec_i128, F64, f64, send_vec_f64);


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
        
        echo "impl_CastVector!(${TYPE}_to_${CAST_TYPE}, \"Vec${UPPER_CASE_TYPE}ToVec${UPPER_CASE_CAST_TYPE}\", $UPPER_CASE_TYPE, recv_vec_$TYPE, $UPPER_CASE_CAST_TYPE, $CAST_TYPE, send_vec_$CAST_TYPE);"
        #echo "${TYPE}_to_${CAST_TYPE}::register(&mut c);"
    done
    
    echo 
done
```
    
*/
