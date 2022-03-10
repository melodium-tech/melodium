
use crate::core::prelude::*;

macro_rules! impl_StreamToBlock {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("flow";$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("data",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("data",Vector,$mel_type,Block)
            ],
            host {
                let input = host.get_input("data");
                let output = host.get_output("data");

                let mut block = Vec::new();

                while let Ok(vec_data) = input.$recv_func().await {

                    block.extend(vec_data);
                }

                let _ = output.$send_func(block).await;
            
                ResultStatus::Ok
            }
        );
    }
}

impl_StreamToBlock!(stream_u8_to_block_vec, "StreamU8ToBlockVecU8", U8, recv_u8, send_vec_u8);
impl_StreamToBlock!(stream_u16_to_block_vec, "StreamU16ToBlockVecU16", U16, recv_u16, send_vec_u16);
impl_StreamToBlock!(stream_u32_to_block_vec, "StreamU32ToBlockVecU32", U32, recv_u32, send_vec_u32);
impl_StreamToBlock!(stream_u64_to_block_vec, "StreamU64ToBlockVecU64", U64, recv_u64, send_vec_u64);
impl_StreamToBlock!(stream_u128_to_block_vec, "StreamU128ToBlockVecU128", U128, recv_u128, send_vec_u128);
impl_StreamToBlock!(stream_i8_to_block_vec, "StreamI8ToBlockVecI8", I8, recv_i8, send_vec_i8);
impl_StreamToBlock!(stream_i16_to_block_vec, "StreamI16ToBlockVecI16", I16, recv_i16, send_vec_i16);
impl_StreamToBlock!(stream_i32_to_block_vec, "StreamI32ToBlockVecI32", I32, recv_i32, send_vec_i32);
impl_StreamToBlock!(stream_i64_to_block_vec, "StreamI64ToBlockVecI64", I64, recv_i64, send_vec_i64);
impl_StreamToBlock!(stream_i128_to_block_vec, "StreamI128ToBlockVecI128", I128, recv_i128, send_vec_i128);
impl_StreamToBlock!(stream_f32_to_block_vec, "StreamF32ToBlockVecF32", F32, recv_f32, send_vec_f32);
impl_StreamToBlock!(stream_f64_to_block_vec, "StreamF64ToBlockVecF64", F64, recv_f64, send_vec_f64);
impl_StreamToBlock!(stream_bool_to_block_vec, "StreamBoolToBlockVecBool", Bool, recv_bool, send_vec_bool);
impl_StreamToBlock!(stream_byte_to_block_vec, "StreamByteToBlockVecByte", Byte, recv_byte, send_vec_byte);
impl_StreamToBlock!(stream_char_to_block_vec, "StreamCharToBlockVecChar", Char, recv_char, send_vec_char);
impl_StreamToBlock!(stream_string_to_block_vec, "StreamStringToBlockVecString", String, recv_string, send_vec_string);


pub fn register(mut c: &mut CollectionPool) {

    stream_u8_to_block_vec::register(&mut c);
    stream_u16_to_block_vec::register(&mut c);
    stream_u32_to_block_vec::register(&mut c);
    stream_u64_to_block_vec::register(&mut c);
    stream_u128_to_block_vec::register(&mut c);
    stream_i8_to_block_vec::register(&mut c);
    stream_i16_to_block_vec::register(&mut c);
    stream_i32_to_block_vec::register(&mut c);
    stream_i64_to_block_vec::register(&mut c);
    stream_i128_to_block_vec::register(&mut c);
    stream_f32_to_block_vec::register(&mut c);
    stream_f64_to_block_vec::register(&mut c);
    stream_bool_to_block_vec::register(&mut c);
    stream_byte_to_block_vec::register(&mut c);
    stream_char_to_block_vec::register(&mut c);
    stream_string_to_block_vec::register(&mut c);
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
    echo "impl_StreamToBlock!(stream_${TYPE}_to_block_vec, \"Stream${UPPER_CASE_TYPE}ToBlockVec${UPPER_CASE_TYPE}\", $UPPER_CASE_TYPE, recv_$TYPE, send_vec_$TYPE);"
    #echo "stream_${TYPE}_to_block_vec::register(&mut c);"

done
```
    
*/
