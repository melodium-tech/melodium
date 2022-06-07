
use crate::core::prelude::*;

macro_rules! impl_BlockScalarFilling {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $mel_value_type:ident, $rust_type:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("filling","scalar";$mel_name),
            models![],
            treatment_sources![],
            parameters![
                parameter!("value",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
            ],
            inputs![
                input!("trigger",Scalar,Void,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type,Stream)
            ],
            host {
                let trigger = host.get_input("trigger");
                let output = host.get_output("value");

                let value = host.get_parameter("value").$mel_value_type();
            
                if let Ok(_) = trigger.recv_one_void().await {

                    let _ = output.$send_func(value).await;
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_BlockScalarFilling!(scalar_block_u8, "StaticBlockU8", U8, u8, u8, send_u8);
impl_BlockScalarFilling!(scalar_block_u16, "StaticBlockU16", U16, u16, u16, send_u16);
impl_BlockScalarFilling!(scalar_block_u32, "StaticBlockU32", U32, u32, u32, send_u32);
impl_BlockScalarFilling!(scalar_block_u64, "StaticBlockU64", U64, u64, u64, send_u64);
impl_BlockScalarFilling!(scalar_block_u128, "StaticBlockU128", U128, u128, u128, send_u128);
impl_BlockScalarFilling!(scalar_block_i8, "StaticBlockI8", I8, i8, i8, send_i8);
impl_BlockScalarFilling!(scalar_block_i16, "StaticBlockI16", I16, i16, i16, send_i16);
impl_BlockScalarFilling!(scalar_block_i32, "StaticBlockI32", I32, i32, i32, send_i32);
impl_BlockScalarFilling!(scalar_block_i64, "StaticBlockI64", I64, i64, i64, send_i64);
impl_BlockScalarFilling!(scalar_block_i128, "StaticBlockI128", I128, i128, i128, send_i128);
impl_BlockScalarFilling!(scalar_block_f32, "StaticBlockF32", F32, f32, f32, send_f32);
impl_BlockScalarFilling!(scalar_block_f64, "StaticBlockF64", F64, f64, f64, send_f64);
impl_BlockScalarFilling!(scalar_block_bool, "StaticBlockBool", Bool, bool, bool, send_bool);
impl_BlockScalarFilling!(scalar_block_byte, "StaticBlockByte", Byte, byte, u8, send_byte);
impl_BlockScalarFilling!(scalar_block_char, "StaticBlockChar", Char, char, char, send_char);
impl_BlockScalarFilling!(scalar_block_string, "StaticBlockString", String, string, String, send_string);


pub fn register(mut c: &mut CollectionPool) {

    scalar_block_u8::register(&mut c);
    scalar_block_u16::register(&mut c);
    scalar_block_u32::register(&mut c);
    scalar_block_u64::register(&mut c);
    scalar_block_u128::register(&mut c);
    scalar_block_i8::register(&mut c);
    scalar_block_i16::register(&mut c);
    scalar_block_i32::register(&mut c);
    scalar_block_i64::register(&mut c);
    scalar_block_i128::register(&mut c);
    scalar_block_f32::register(&mut c);
    scalar_block_f64::register(&mut c);
    scalar_block_bool::register(&mut c);
    scalar_block_byte::register(&mut c);
    scalar_block_char::register(&mut c);
    scalar_block_string::register(&mut c);
}
