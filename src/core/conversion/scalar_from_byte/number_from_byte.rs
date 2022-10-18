
use crate::core::prelude::*;

macro_rules! impl_NumberFromByte {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $rust_type:ident, $size:expr, $v:ident $decl:expr, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","scalar";$mel_name),
            formatdoc!(r"Convert stream of `Vec<byte>` into `{type}`.

            Each received `byte` vector try to be converted into `{type}`, and if valid is sent as `value`. If the incoming vector 
            is not valid for representing a `{type}` (i.e. not right size or invalid coding) it is refused and sent through `reject`.", type = stringify!($rust_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("data",Vector,Byte,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type,Stream),
                output!("reject",Vector,Byte,Stream)
            ],
            host {
                let input = host.get_input("data");
                let accept = host.get_output("value");
                let reject = host.get_output("reject");

                let mut accepted_op = true;
                let mut rejected_op = true;
            
                'main: while let Ok(vectors) = input.recv_vec_byte().await {
            
                    for $v in vectors {
                        if $v.len() == $size {

                            let num = $rust_type::from_be_bytes($decl);

                            if let Err(_) = accept.$send_func(num).await {
                                // If we cannot send anymore on accepted, we note it,
                                // and check if rejected is still valid, else just terminate.
                                accepted_op = false;
                                if !rejected_op {
                                    break 'main;
                                }
                            }
                        }
                        else {
                            if let Err(_) = reject.send_vec_byte($v).await {
                                // If we cannot send anymore on rejected, we note it,
                                // and check if accepted is still valid, else just terminate.
                                rejected_op = false;
                                if !accepted_op {
                                    break 'main;
                                }
                            }
                        }
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_NumberFromByte!(u8_from_byte, "U8FromByte", U8, u8, 1, v [v[0]], send_u8);
impl_NumberFromByte!(u16_from_byte, "U16FromByte", U16, u16, 2, v [v[0],v[1]], send_u16);
impl_NumberFromByte!(u32_from_byte, "U32FromByte", U32, u32, 4, v [v[0],v[1],v[2],v[3]], send_u32);
impl_NumberFromByte!(u64_from_byte, "U64FromByte", U64, u64, 8, v [v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7]], send_u64);
impl_NumberFromByte!(u128_from_byte, "U128FromByte", U128, u128, 16, v [v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7],v[8],v[9],v[10],v[11],v[12],v[13],v[14],v[15]], send_u128);
impl_NumberFromByte!(i8_from_byte, "I8FromByte", I8, i8, 1, v [v[0]], send_i8);
impl_NumberFromByte!(i16_from_byte, "I16FromByte", I16, i16, 2, v [v[0],v[1]], send_i16);
impl_NumberFromByte!(i32_from_byte, "I32FromByte", I32, i32, 4, v [v[0],v[1],v[2],v[3]], send_i32);
impl_NumberFromByte!(i64_from_byte, "I64FromByte", I64, i64, 8, v [v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7]], send_i64);
impl_NumberFromByte!(i128_from_byte, "I128FromByte", I128, i128, 16, v [v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7],v[8],v[9],v[10],v[11],v[12],v[13],v[14],v[15]], send_i128);
impl_NumberFromByte!(f32_from_byte, "F32FromByte", F32, f32, 4, v [v[0],v[1],v[2],v[3]], send_f32);
impl_NumberFromByte!(f64_from_byte, "F64FromByte", F64, f64, 8, v [v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7]], send_f64);

pub fn register(mut c: &mut CollectionPool) {

    u8_from_byte::register(&mut c);
    u16_from_byte::register(&mut c);
    u32_from_byte::register(&mut c);
    u64_from_byte::register(&mut c);
    u128_from_byte::register(&mut c);
    i8_from_byte::register(&mut c);
    i16_from_byte::register(&mut c);
    i32_from_byte::register(&mut c);
    i64_from_byte::register(&mut c);
    i128_from_byte::register(&mut c);
    f32_from_byte::register(&mut c);
    f64_from_byte::register(&mut c);
}
