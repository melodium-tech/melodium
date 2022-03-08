
use crate::core::prelude::*;

macro_rules! impl_AddScalar {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("arithmetic","scalar";$mel_name),
            models![],
            treatment_sources![],
            parameters![
                parameter!("add",Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
            ],
            inputs![
                input!("value",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");

                let add = host.get_parameter("add").$mel_value_type();
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v + add).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_AddScalar!(add_u8, "AddU8", U8, u8, u8, recv_u8, send_multiple_u8);
impl_AddScalar!(add_u16, "AddU16", U16, u16, u16, recv_u16, send_multiple_u16);
impl_AddScalar!(add_u32, "AddU32", U32, u32, u32, recv_u32, send_multiple_u32);
impl_AddScalar!(add_u64, "AddU64", U64, u64, u64, recv_u64, send_multiple_u64);
impl_AddScalar!(add_u128, "AddU128", U128, u128, u128, recv_u128, send_multiple_u128);
impl_AddScalar!(add_i8, "AddI8", I8, i8, i8, recv_i8, send_multiple_i8);
impl_AddScalar!(add_i16, "AddI16", I16, i16, i16, recv_i16, send_multiple_i16);
impl_AddScalar!(add_i32, "AddI32", I32, i32, i32, recv_i32, send_multiple_i32);
impl_AddScalar!(add_i64, "AddI64", I64, i64, i64, recv_i64, send_multiple_i64);
impl_AddScalar!(add_i128, "AddI128", I128, i128, i128, recv_i128, send_multiple_i128);
impl_AddScalar!(add_f32, "AddF32", F32, f32, f32, recv_f32, send_multiple_f32);
impl_AddScalar!(add_f64, "AddF64", F64, f64, f64, recv_f64, send_multiple_f64);

pub fn register(mut c: &mut CollectionPool) {

    add_u8::register(&mut c);
    add_u16::register(&mut c);
    add_u32::register(&mut c);
    add_u64::register(&mut c);
    add_u128::register(&mut c);
    add_i8::register(&mut c);
    add_i16::register(&mut c);
    add_i32::register(&mut c);
    add_i64::register(&mut c);
    add_i128::register(&mut c);
    add_f32::register(&mut c);
    add_f64::register(&mut c);
}

/*
    FOR DEVELOPERS

The lines above can be regenerated as will using the following script:

```
#!/bin/bash

TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64"

for TYPE in $TYPES
do
    UPPER_CASE_TYPE=${TYPE^}
    #echo "impl_AddScalar!(add_${TYPE}, \"Add${UPPER_CASE_TYPE}\", $UPPER_CASE_TYPE, $TYPE, $TYPE, recv_$TYPE, send_multiple_$TYPE);"
    echo "add_${TYPE}::register(&mut c);"

done
```
*/

