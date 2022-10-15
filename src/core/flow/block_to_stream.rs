
use crate::core::prelude::*;

macro_rules! impl_BlockToStream {
    ($mod:ident, $mel_name:expr, $mel_struct:ident, $mel_type:ident, $type:expr, $recv_func:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("flow";$mel_name),
            format!(r#"Stream a block of `{}`.

            The received block is sent as streamed value, then the stream is closed.
            
            ```mermaid
            graph LR
                T(StreamBlock)
                B["〈🟦〉"] -->|block| T
                
                T -->|stream| O["🟦"]
            
                style B fill:#ffff,stroke:#ffff
                style O fill:#ffff,stroke:#ffff
            ```"#, $type),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("data",$mel_struct,$mel_type,Block)
            ],
            outputs![
                output!("data",$mel_struct,$mel_type,Stream)
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

impl_BlockToStream!(block_void_to_stream, "BlockVoidToStream", Scalar, Void, "void", recv_void, send_void);
impl_BlockToStream!(block_u8_to_stream, "BlockU8ToStream", Scalar, U8, "u8", recv_u8, send_u8);
impl_BlockToStream!(block_u16_to_stream, "BlockU16ToStream", Scalar, U16, "u16", recv_u16, send_u16);
impl_BlockToStream!(block_u32_to_stream, "BlockU32ToStream", Scalar, U32, "u32", recv_u32, send_u32);
impl_BlockToStream!(block_u64_to_stream, "BlockU64ToStream", Scalar, U64, "u64", recv_u64, send_u64);
impl_BlockToStream!(block_u128_to_stream, "BlockU128ToStream", Scalar, U128, "u128", recv_u128, send_u128);
impl_BlockToStream!(block_i8_to_stream, "BlockI8ToStream", Scalar, I8, "i8", recv_i8, send_i8);
impl_BlockToStream!(block_i16_to_stream, "BlockI16ToStream", Scalar, I16, "i16", recv_i16, send_i16);
impl_BlockToStream!(block_i32_to_stream, "BlockI32ToStream", Scalar, I32, "i32", recv_i32, send_i32);
impl_BlockToStream!(block_i64_to_stream, "BlockI64ToStream", Scalar, I64, "i64", recv_i64, send_i64);
impl_BlockToStream!(block_i128_to_stream, "BlockI128ToStream", Scalar, I128, "i128", recv_i128, send_i128);
impl_BlockToStream!(block_f32_to_stream, "BlockF32ToStream", Scalar, F32, "f32", recv_f32, send_f32);
impl_BlockToStream!(block_f64_to_stream, "BlockF64ToStream", Scalar, F64, "f64", recv_f64, send_f64);
impl_BlockToStream!(block_bool_to_stream, "BlockBoolToStream", Scalar, Bool, "bool", recv_bool, send_bool);
impl_BlockToStream!(block_byte_to_stream, "BlockByteToStream", Scalar, Byte, "byte", recv_byte, send_byte);
impl_BlockToStream!(block_char_to_stream, "BlockCharToStream", Scalar, Char, "char", recv_char, send_char);
impl_BlockToStream!(block_string_to_stream, "BlockStringToStream", Scalar, String, "string", recv_string, send_string);

impl_BlockToStream!(block_vec_void_to_stream, "BlockVecVoidToStream", Vector, Void, "Vec<void>", recv_vec_void, send_vec_void);
impl_BlockToStream!(block_vec_u8_to_stream, "BlockVecU8ToStream", Vector, U8, "Vec<u8>", recv_vec_u8, send_vec_u8);
impl_BlockToStream!(block_vec_u16_to_stream, "BlockVecU16ToStream", Vector, U16, "Vec<u16>", recv_vec_u16, send_vec_u16);
impl_BlockToStream!(block_vec_u32_to_stream, "BlockVecU32ToStream", Vector, U32, "Vec<u32>", recv_vec_u32, send_vec_u32);
impl_BlockToStream!(block_vec_u64_to_stream, "BlockVecU64ToStream", Vector, U64, "Vec<u64>", recv_vec_u64, send_vec_u64);
impl_BlockToStream!(block_vec_u128_to_stream, "BlockVecU128ToStream", Vector, U128, "Vec<u128>", recv_vec_u128, send_vec_u128);
impl_BlockToStream!(block_vec_i8_to_stream, "BlockVecI8ToStream", Vector, I8, "Vec<i8>", recv_vec_i8, send_vec_i8);
impl_BlockToStream!(block_vec_i16_to_stream, "BlockVecI16ToStream", Vector, I16, "Vec<i16>", recv_vec_i16, send_vec_i16);
impl_BlockToStream!(block_vec_i32_to_stream, "BlockVecI32ToStream", Vector, I32, "Vec<i32>", recv_vec_i32, send_vec_i32);
impl_BlockToStream!(block_vec_i64_to_stream, "BlockVecI64ToStream", Vector, I64, "Vec<i64>", recv_vec_i64, send_vec_i64);
impl_BlockToStream!(block_vec_i128_to_stream, "BlockVecI128ToStream", Vector, I128, "Vec<i128>", recv_vec_i128, send_vec_i128);
impl_BlockToStream!(block_vec_f32_to_stream, "BlockVecF32ToStream", Vector, F32, "Vec<f32>", recv_vec_f32, send_vec_f32);
impl_BlockToStream!(block_vec_f64_to_stream, "BlockVecF64ToStream", Vector, F64, "Vec<f64>", recv_vec_f64, send_vec_f64);
impl_BlockToStream!(block_vec_bool_to_stream, "BlockVecBoolToStream", Vector, Bool, "Vec<bool>", recv_vec_bool, send_vec_bool);
impl_BlockToStream!(block_vec_byte_to_stream, "BlockVecByteToStream", Vector, Byte, "Vec<byte>", recv_vec_byte, send_vec_byte);
impl_BlockToStream!(block_vec_char_to_stream, "BlockVecCharToStream", Vector, Char, "Vec<char>", recv_vec_char, send_vec_char);
impl_BlockToStream!(block_vec_string_to_stream, "BlockVecStringToStream", Vector, String, "Vec<string>", recv_vec_string, send_vec_string);

pub fn register(mut c: &mut CollectionPool) {

    block_void_to_stream::register(&mut c);
    block_u8_to_stream::register(&mut c);
    block_u16_to_stream::register(&mut c);
    block_u32_to_stream::register(&mut c);
    block_u64_to_stream::register(&mut c);
    block_u128_to_stream::register(&mut c);
    block_i8_to_stream::register(&mut c);
    block_i16_to_stream::register(&mut c);
    block_i32_to_stream::register(&mut c);
    block_i64_to_stream::register(&mut c);
    block_i128_to_stream::register(&mut c);
    block_f32_to_stream::register(&mut c);
    block_f64_to_stream::register(&mut c);
    block_bool_to_stream::register(&mut c);
    block_byte_to_stream::register(&mut c);
    block_char_to_stream::register(&mut c);
    block_string_to_stream::register(&mut c);

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

TYPES="void u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 bool byte char string"

for TYPE in $TYPES
do
    UPPER_CASE_TYPE=${TYPE^}
    echo "impl_BlockToStream!(block_${TYPE}_to_stream, \"Block${UPPER_CASE_TYPE}ToStream\", Scalar, $UPPER_CASE_TYPE, \"$TYPE\", recv_$TYPE, send_$TYPE);"
    #echo "block_vec_${TYPE}_to_stream::register(&mut c);"

done
```
    
*/
