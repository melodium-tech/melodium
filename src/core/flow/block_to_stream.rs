
use crate::core::prelude::*;

macro_rules! impl_BlockToStream {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("flow";$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("data",Vector,$mel_type,Block)
            ],
            outputs![
                output!("data",Scalar,$mel_type,Stream)
            ],
            host {
                let input = host.get_input("data");
                let output = host.get_output("data");

                if let Ok(vec_data) = input.$recv_func().await {
                    let _ = output.$send_func(vec_data.get(0).unwrap().clone()).await;
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_BlockToStream!(block_vec_void_to_stream, "BlockVecVoidToStreamVoid", Void, recv_vec_void, send_multiple_void);
impl_BlockToStream!(block_vec_u8_to_stream, "BlockVecU8ToStreamU8", U8, recv_vec_u8, send_multiple_u8);
impl_BlockToStream!(block_vec_u16_to_stream, "BlockVecU16ToStreamU16", U16, recv_vec_u16, send_multiple_u16);
impl_BlockToStream!(block_vec_u32_to_stream, "BlockVecU32ToStreamU32", U32, recv_vec_u32, send_multiple_u32);
impl_BlockToStream!(block_vec_u64_to_stream, "BlockVecU64ToStreamU64", U64, recv_vec_u64, send_multiple_u64);
impl_BlockToStream!(block_vec_u128_to_stream, "BlockVecU128ToStreamU128", U128, recv_vec_u128, send_multiple_u128);
impl_BlockToStream!(block_vec_i8_to_stream, "BlockVecI8ToStreamI8", I8, recv_vec_i8, send_multiple_i8);
impl_BlockToStream!(block_vec_i16_to_stream, "BlockVecI16ToStreamI16", I16, recv_vec_i16, send_multiple_i16);
impl_BlockToStream!(block_vec_i32_to_stream, "BlockVecI32ToStreamI32", I32, recv_vec_i32, send_multiple_i32);
impl_BlockToStream!(block_vec_i64_to_stream, "BlockVecI64ToStreamI64", I64, recv_vec_i64, send_multiple_i64);
impl_BlockToStream!(block_vec_i128_to_stream, "BlockVecI128ToStreamI128", I128, recv_vec_i128, send_multiple_i128);
impl_BlockToStream!(block_vec_f32_to_stream, "BlockVecF32ToStreamF32", F32, recv_vec_f32, send_multiple_f32);
impl_BlockToStream!(block_vec_f64_to_stream, "BlockVecF64ToStreamF64", F64, recv_vec_f64, send_multiple_f64);
impl_BlockToStream!(block_vec_bool_to_stream, "BlockVecBoolToStreamBool", Bool, recv_vec_bool, send_multiple_bool);
impl_BlockToStream!(block_vec_byte_to_stream, "BlockVecByteToStreamByte", Byte, recv_vec_byte, send_multiple_byte);
impl_BlockToStream!(block_vec_char_to_stream, "BlockVecCharToStreamChar", Char, recv_vec_char, send_multiple_char);
impl_BlockToStream!(block_vec_string_to_stream, "BlockVecStringToStreamString", String, recv_vec_string, send_multiple_string);

pub fn register(mut c: &mut CollectionPool) {

    block_vec_void_to_stream::register(&mut c);
    block_vec_u8_to_stream::register(&mut c);
    block_vec_u16_to_stream::register(&mut c);
    block_vec_u32_to_stream::register(&mut c);
    block_vec_u64_to_stream::register(&mut c);
    block_vec_u128_to_stream::register(&mut c);
    block_vec_i8_to_stream::register(&mut c);
    block_vec_i16_to_stream::register(&mut c);
    block_vec_i32_to_stream::register(&mut c);
    block_vec_i64_to_stream::register(&mut c);
    block_vec_i128_to_stream::register(&mut c);
    block_vec_f32_to_stream::register(&mut c);
    block_vec_f64_to_stream::register(&mut c);
    block_vec_bool_to_stream::register(&mut c);
    block_vec_byte_to_stream::register(&mut c);
    block_vec_char_to_stream::register(&mut c);
    block_vec_string_to_stream::register(&mut c);
}

/*
    FOR DEVELOPERS

The lines above can be regenerated as will using the following script:

```
#!/bin/bash

TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 bool byte char string"

for TYPE in $TYPES
do
    UPPER_CASE_TYPE=${TYPE^}
    echo "impl_BlockToStream!(block_vec_${TYPE}_to_stream, \"BlockVec${UPPER_CASE_TYPE}ToStream${UPPER_CASE_TYPE}\", $UPPER_CASE_TYPE, recv_vec_$TYPE, send_multiple_$TYPE);"
    #echo "block_vec_${TYPE}_to_stream::register(&mut c);"

done
```
    
*/
