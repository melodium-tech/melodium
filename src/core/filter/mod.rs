
use crate::core::prelude::*;

macro_rules! impl_Filter {
    ($mod_name:ident, $mel_type_name:expr, $mel_type_up:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($mod_name,
            core_identifier!("filter","scalar",$mel_type_name;"Filter"),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type_up,Stream),
                input!("decision",Scalar,Bool,Stream)
            ],
            outputs![
                output!("accepted",Scalar,$mel_type_up,Stream),
                output!("rejected",Scalar,$mel_type_up,Stream)
            ],
            host {
                let input_value = host.get_input("value");
                let input_decision = host.get_input("decision");

                let output_accepted = host.get_output("accepted");
                let output_rejected = host.get_output("rejected");

                let mut accepted_op = true;
                let mut rejected_op = true;
            
                while let (Ok(value), Ok(decision)) = futures::join!(input_value.$recv_func(), input_decision.recv_one_bool()) {

                    if decision {
                        if let Err(_) = output_accepted.$send_func(value).await {
                            // If we cannot send anymore on accepted, we note it,
                            // and check if rejected is still valid, else just terminate.
                            accepted_op = false;
                            if !rejected_op {
                                break;
                            }
                        }
                    }
                    else {
                        if let Err(_) = output_rejected.$send_func(value).await {
                            // If we cannot send anymore on rejected, we note it,
                            // and check if accepted is still valid, else just terminate.
                            rejected_op = false;
                            if !accepted_op {
                                break;
                            }
                        }
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_Filter!(scalar_void, "void", Void, recv_one_void, send_void);

impl_Filter!(scalar_u8, "u8", U8, recv_one_u8, send_u8);
impl_Filter!(scalar_u16, "u16", U16, recv_one_u16, send_u16);
impl_Filter!(scalar_u32, "u32", U32, recv_one_u32, send_u32);
impl_Filter!(scalar_u64, "u64", U64, recv_one_u64, send_u64);
impl_Filter!(scalar_u128, "u128", U128, recv_one_u128, send_u128);

impl_Filter!(scalar_i8, "i8", I8, recv_one_i8, send_i8);
impl_Filter!(scalar_i16, "i16", I16, recv_one_i16, send_i16);
impl_Filter!(scalar_i32, "i32", I32, recv_one_i32, send_i32);
impl_Filter!(scalar_i64, "i64", I64, recv_one_i64, send_i64);
impl_Filter!(scalar_i128, "i128", I128, recv_one_i128, send_i128);

impl_Filter!(scalar_f32, "f32", F32, recv_one_f32, send_f32);
impl_Filter!(scalar_f64, "f64", F64, recv_one_f64, send_f64);

impl_Filter!(scalar_bool, "bool", Bool, recv_one_bool, send_bool);
impl_Filter!(scalar_byte, "byte", Byte, recv_one_byte, send_byte);

impl_Filter!(scalar_char, "char", Char, recv_one_char, send_char);
impl_Filter!(scalar_string, "string", String, recv_one_string, send_string);

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
}

