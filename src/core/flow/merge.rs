
use crate::core::prelude::*;

macro_rules! impl_MergeScalar {
    ($mod_name:ident, $mel_type_name:expr, $mel_type_up:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($mod_name,
            core_identifier!("merge","scalar",$mel_type_name;"Merge"),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("a",Scalar,$mel_type_up,Stream),
                input!("b",Scalar,$mel_type_up,Stream),
                input!("select",Scalar,Bool,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type_up,Stream)
            ],
            host {
                let input_a = host.get_input("a");
                let input_b = host.get_input("b");
                let input_decision = host.get_input("select");

                let output = host.get_output("value");

                while let Ok(select) = input_decision.recv_one_bool().await {

                    let value;
                    if select {
                        if let Ok(v) = input_a.$recv_func().await {
                            value = v;
                        }
                        else {
                            break;
                        }
                    }
                    else {
                        if let Ok(v) = input_b.$recv_func().await {
                            value = v;
                        }
                        else {
                            break;
                        }
                    }

                    ok_or_break!(output.$send_func(value).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}


macro_rules! impl_MergeVector {
    ($mod_name:ident, $mel_type_name:expr, $mel_type_up:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($mod_name,
            core_identifier!("merge","vector",$mel_type_name;"Merge"),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("a",Vector,$mel_type_up,Stream),
                input!("b",Vector,$mel_type_up,Stream),
                input!("select",Scalar,Bool,Stream)
            ],
            outputs![
                output!("value",Vector,$mel_type_up,Stream)
            ],
            host {
                let input_a = host.get_input("a");
                let input_b = host.get_input("b");
                let input_decision = host.get_input("select");

                let output = host.get_output("value");

                while let Ok(select) = input_decision.recv_one_bool().await {

                    let value;
                    if select {
                        if let Ok(v) = input_a.$recv_func().await {
                            value = v;
                        }
                        else {
                            break;
                        }
                    }
                    else {
                        if let Ok(v) = input_b.$recv_func().await {
                            value = v;
                        }
                        else {
                            break;
                        }
                    }

                    ok_or_break!(output.$send_func(value).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_MergeScalar!(scalar_void, "void", Void, recv_one_void, send_void);

impl_MergeScalar!(scalar_u8, "u8", U8, recv_one_u8, send_u8);
impl_MergeScalar!(scalar_u16, "u16", U16, recv_one_u16, send_u16);
impl_MergeScalar!(scalar_u32, "u32", U32, recv_one_u32, send_u32);
impl_MergeScalar!(scalar_u64, "u64", U64, recv_one_u64, send_u64);
impl_MergeScalar!(scalar_u128, "u128", U128, recv_one_u128, send_u128);

impl_MergeScalar!(scalar_i8, "i8", I8, recv_one_i8, send_i8);
impl_MergeScalar!(scalar_i16, "i16", I16, recv_one_i16, send_i16);
impl_MergeScalar!(scalar_i32, "i32", I32, recv_one_i32, send_i32);
impl_MergeScalar!(scalar_i64, "i64", I64, recv_one_i64, send_i64);
impl_MergeScalar!(scalar_i128, "i128", I128, recv_one_i128, send_i128);

impl_MergeScalar!(scalar_f32, "f32", F32, recv_one_f32, send_f32);
impl_MergeScalar!(scalar_f64, "f64", F64, recv_one_f64, send_f64);

impl_MergeScalar!(scalar_bool, "bool", Bool, recv_one_bool, send_bool);
impl_MergeScalar!(scalar_byte, "byte", Byte, recv_one_byte, send_byte);

impl_MergeScalar!(scalar_char, "char", Char, recv_one_char, send_char);
impl_MergeScalar!(scalar_string, "string", String, recv_one_string, send_string);



impl_MergeVector!(vector_void, "void", Void, recv_one_vec_void, send_vec_void);

impl_MergeVector!(vector_u8, "u8", U8, recv_one_vec_u8, send_vec_u8);
impl_MergeVector!(vector_u16, "u16", U16, recv_one_vec_u16, send_vec_u16);
impl_MergeVector!(vector_u32, "u32", U32, recv_one_vec_u32, send_vec_u32);
impl_MergeVector!(vector_u64, "u64", U64, recv_one_vec_u64, send_vec_u64);
impl_MergeVector!(vector_u128, "u128", U128, recv_one_vec_u128, send_vec_u128);

impl_MergeVector!(vector_i8, "i8", I8, recv_one_vec_i8, send_vec_i8);
impl_MergeVector!(vector_i16, "i16", I16, recv_one_vec_i16, send_vec_i16);
impl_MergeVector!(vector_i32, "i32", I32, recv_one_vec_i32, send_vec_i32);
impl_MergeVector!(vector_i64, "i64", I64, recv_one_vec_i64, send_vec_i64);
impl_MergeVector!(vector_i128, "i128", I128, recv_one_vec_i128, send_vec_i128);

impl_MergeVector!(vector_f32, "f32", F32, recv_one_vec_f32, send_vec_f32);
impl_MergeVector!(vector_f64, "f64", F64, recv_one_vec_f64, send_vec_f64);

impl_MergeVector!(vector_bool, "bool", Bool, recv_one_vec_bool, send_vec_bool);
impl_MergeVector!(vector_byte, "byte", Byte, recv_one_vec_byte, send_vec_byte);

impl_MergeVector!(vector_char, "char", Char, recv_one_vec_char, send_vec_char);
impl_MergeVector!(vector_string, "string", String, recv_one_vec_string, send_vec_string);

pub fn register(mut c: &mut CollectionPool) {

    scalar_void::register(&mut c);
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

    vector_void::register(&mut c);
    vector_u8::register(&mut c);
    vector_u16::register(&mut c);
    vector_u32::register(&mut c);
    vector_u64::register(&mut c);
    vector_u128::register(&mut c);
    vector_i8::register(&mut c);
    vector_i16::register(&mut c);
    vector_i32::register(&mut c);
    vector_i64::register(&mut c);
    vector_i128::register(&mut c);
    vector_f32::register(&mut c);
    vector_f64::register(&mut c);
    vector_bool::register(&mut c);
    vector_byte::register(&mut c);
    vector_char::register(&mut c);
    vector_string::register(&mut c);
}
