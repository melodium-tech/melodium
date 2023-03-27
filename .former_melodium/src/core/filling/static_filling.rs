
use crate::core::prelude::*;

macro_rules! impl_ScalarFilling {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $mel_value_type:ident, $rust_type:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("filling","scalar";$mel_name),
            formatdoc!(r#"Fill an input `void` stream with scalar static `{type}` values.

            ```mermaid
            graph LR
                T("StaticFill (value=🟧)")
                B["… 🟦 🟦 🟦 …"] -->|pattern| T
                
                T -->|value| O["… 🟧 🟧 🟧 …"]
            
                style B fill:#ffff,stroke:#ffff
                style O fill:#ffff,stroke:#ffff
            ```"#, type = stringify!($mel_type)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("value",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
            ],
            inputs![
                input!("pattern",Scalar,Void,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type,Stream)
            ],
            host {
                let pattern = host.get_input("pattern");
                let output = host.get_output("value");

                let value = host.get_parameter("value").$mel_value_type();
            
                while let Ok(values) = pattern.recv_void().await {
            
                    let generated = vec![value.clone(); values.len()];

                    ok_or_break!(output.$send_func(generated).await);

                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_ScalarVecFilling {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $mel_value_type:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("filling","scalar";$mel_name),
            formatdoc!(r#"Fill an input `void` stream with vector static `{type}` values.

            ```mermaid
            graph LR
                T("StaticVecFill (value=［🟧 🟧 🟧］)")
                B["… 🟦 🟦 🟦 …"] -->|pattern| T
                
                T -->|value| O["… ［🟧 🟧 🟧］［🟧 🟧 🟧］［🟧 🟧 🟧］ …"]
            
                style B fill:#ffff,stroke:#ffff
                style O fill:#ffff,stroke:#ffff
            ```"#, type = stringify!($mel_type)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("value",Var,Vector,$mel_type,None)
            ],
            inputs![
                input!("pattern",Scalar,Void,Stream)
            ],
            outputs![
                output!("value",Vector,$mel_type,Stream)
            ],
            host {
                let pattern = host.get_input("pattern");
                let output = host.get_output("value");

                let value = host.get_parameter("value").$mel_value_type();
            
                'main: while let Ok(values) = pattern.recv_void().await {
            
                    for _ in values {
                        ok_or_break!('main, output.$send_func(value.clone()).await);
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_ScalarFilling!(scalar_u8, "StaticU8", U8, u8, u8, send_multiple_u8);
impl_ScalarFilling!(scalar_u16, "StaticU16", U16, u16, u16, send_multiple_u16);
impl_ScalarFilling!(scalar_u32, "StaticU32", U32, u32, u32, send_multiple_u32);
impl_ScalarFilling!(scalar_u64, "StaticU64", U64, u64, u64, send_multiple_u64);
impl_ScalarFilling!(scalar_u128, "StaticU128", U128, u128, u128, send_multiple_u128);
impl_ScalarFilling!(scalar_i8, "StaticI8", I8, i8, i8, send_multiple_i8);
impl_ScalarFilling!(scalar_i16, "StaticI16", I16, i16, i16, send_multiple_i16);
impl_ScalarFilling!(scalar_i32, "StaticI32", I32, i32, i32, send_multiple_i32);
impl_ScalarFilling!(scalar_i64, "StaticI64", I64, i64, i64, send_multiple_i64);
impl_ScalarFilling!(scalar_i128, "StaticI128", I128, i128, i128, send_multiple_i128);
impl_ScalarFilling!(scalar_f32, "StaticF32", F32, f32, f32, send_multiple_f32);
impl_ScalarFilling!(scalar_f64, "StaticF64", F64, f64, f64, send_multiple_f64);
impl_ScalarFilling!(scalar_bool, "StaticBool", Bool, bool, bool, send_multiple_bool);
impl_ScalarFilling!(scalar_byte, "StaticByte", Byte, byte, u8, send_multiple_byte);
impl_ScalarFilling!(scalar_char, "StaticChar", Char, char, char, send_multiple_char);
impl_ScalarFilling!(scalar_string, "StaticString", String, string, String, send_multiple_string);

impl_ScalarVecFilling!(scalar_vec_void, "StaticVecVoid", Void, vec_void, send_vec_void);
impl_ScalarVecFilling!(scalar_vec_u8, "StaticVecU8", U8, vec_u8, send_vec_u8);
impl_ScalarVecFilling!(scalar_vec_u16, "StaticVecU16", U16, vec_u16, send_vec_u16);
impl_ScalarVecFilling!(scalar_vec_u32, "StaticVecU32", U32, vec_u32, send_vec_u32);
impl_ScalarVecFilling!(scalar_vec_u64, "StaticVecU64", U64, vec_u64, send_vec_u64);
impl_ScalarVecFilling!(scalar_vec_u128, "StaticVecU128", U128, vec_u128, send_vec_u128);
impl_ScalarVecFilling!(scalar_vec_i8, "StaticVecI8", I8, vec_i8, send_vec_i8);
impl_ScalarVecFilling!(scalar_vec_i16, "StaticVecI16", I16, vec_i16, send_vec_i16);
impl_ScalarVecFilling!(scalar_vec_i32, "StaticVecI32", I32, vec_i32, send_vec_i32);
impl_ScalarVecFilling!(scalar_vec_i64, "StaticVecI64", I64, vec_i64, send_vec_i64);
impl_ScalarVecFilling!(scalar_vec_i128, "StaticVecI128", I128, vec_i128, send_vec_i128);
impl_ScalarVecFilling!(scalar_vec_f32, "StaticVecF32", F32, vec_f32, send_vec_f32);
impl_ScalarVecFilling!(scalar_vec_f64, "StaticVecF64", F64, vec_f64, send_vec_f64);
impl_ScalarVecFilling!(scalar_vec_bool, "StaticVecBool", Bool, vec_bool, send_vec_bool);
impl_ScalarVecFilling!(scalar_vec_byte, "StaticVecByte", Byte, vec_byte, send_vec_byte);
impl_ScalarVecFilling!(scalar_vec_char, "StaticVecChar", Char, vec_char, send_vec_char);
impl_ScalarVecFilling!(scalar_vec_string, "StaticVecString", String, vec_string, send_vec_string);


pub fn register(mut c: &mut CollectionPool) {

    scalar_u8::register(&mut c);
    scalar_u16::register(&mut c);
    scalar_u32::register(&mut c);
    scalar_u64::register(&mut c);
    scalar_u128::register(&mut c);
    scalar_i8::register(&mut c);
    scalar_i16::register(&mut c);
    scalar_i32::register(&mut c);
    scalar_i64::register(&mut c);
    scalar_i128::register(&mut c);
    scalar_f32::register(&mut c);
    scalar_f64::register(&mut c);
    scalar_bool::register(&mut c);
    scalar_byte::register(&mut c);
    scalar_char::register(&mut c);
    scalar_string::register(&mut c);

    scalar_vec_void::register(&mut c);
    scalar_vec_u8::register(&mut c);
    scalar_vec_u16::register(&mut c);
    scalar_vec_u32::register(&mut c);
    scalar_vec_u64::register(&mut c);
    scalar_vec_u128::register(&mut c);
    scalar_vec_i8::register(&mut c);
    scalar_vec_i16::register(&mut c);
    scalar_vec_i32::register(&mut c);
    scalar_vec_i64::register(&mut c);
    scalar_vec_i128::register(&mut c);
    scalar_vec_f32::register(&mut c);
    scalar_vec_f64::register(&mut c);
    scalar_vec_bool::register(&mut c);
    scalar_vec_byte::register(&mut c);
    scalar_vec_char::register(&mut c);
    scalar_vec_string::register(&mut c);
}
