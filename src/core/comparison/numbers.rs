
use crate::core::prelude::*;

macro_rules! impl_Comparison {
    ($mel_name:expr, $mel_type_name:expr, $mel_type:ident, $comp_func:ident, $recv_func:ident, $send_func:ident) => {
        
        treatment!($comp_func,
            core_identifier!("logic","scalar",$mel_type_name;$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("a",Scalar,$mel_type,Stream),
                input!("b",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("is",Scalar,Bool,Stream)
            ],
            host {
                let input_a = host.get_input("a");
                let input_b = host.get_input("b");
                let output = host.get_output("is");
            
                while let (Ok(a), Ok(b)) = futures::join!(input_a.$recv_func(), input_b.$recv_func()) {

                    ok_or_break!(output.$send_func(a.$comp_func(&b)).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_comp_function {
    ($mel_name:expr, $mel_type_name:expr, $mel_type:ident, $mel_value_type:ident, $comp_func:ident) => {
        fn $comp_func() -> Arc<CoreFunctionDescriptor> {

            fn func(params: Vec<Value>) -> Value {
                Value::Bool(params[0].clone().$mel_value_type().$comp_func(&params[1].clone().$mel_value_type()))
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("logic","scalar",$mel_type_name;&format!("|{}", $mel_name)),
                parameters![
                    parameter!("a", Scalar, $mel_type, None),
                    parameter!("b", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, Bool),
                func
            )
        }
    };
}

macro_rules! integer {
    ($mel_type_name:expr, $mel_type:ident, $mel_value_type:ident, $recv_func:ident) => {
        mod $mel_value_type {
            use crate::core::prelude::*;

            impl_Comparison!("GreaterThan", $mel_type_name, $mel_type, gt, $recv_func, send_bool);
            impl_comp_function!("gt", $mel_type_name, $mel_type, $mel_value_type, gt);
            impl_Comparison!("GreaterEqual", $mel_type_name, $mel_type, ge, $recv_func, send_bool);
            impl_comp_function!("ge", $mel_type_name, $mel_type, $mel_value_type, ge);
            impl_Comparison!("LowerThan", $mel_type_name, $mel_type, lt, $recv_func, send_bool);
            impl_comp_function!("lt", $mel_type_name, $mel_type, $mel_value_type, lt);
            impl_Comparison!("LowerEqual", $mel_type_name, $mel_type, le, $recv_func, send_bool);
            impl_comp_function!("le", $mel_type_name, $mel_type, $mel_value_type, le);
            impl_Comparison!("Equal", $mel_type_name, $mel_type, eq, $recv_func, send_bool);
            impl_comp_function!("eq", $mel_type_name, $mel_type, $mel_value_type, eq);
            impl_Comparison!("NotEqual", $mel_type_name, $mel_type, ne, $recv_func, send_bool);
            impl_comp_function!("ne", $mel_type_name, $mel_type, $mel_value_type, ne);

            pub fn register(mut c: &mut CollectionPool) {

                gt::register(&mut c);
                c.functions.insert(&(gt() as Arc<dyn FunctionDescriptor>));
                ge::register(&mut c);
                c.functions.insert(&(ge() as Arc<dyn FunctionDescriptor>));
                lt::register(&mut c);
                c.functions.insert(&(lt() as Arc<dyn FunctionDescriptor>));
                le::register(&mut c);
                c.functions.insert(&(le() as Arc<dyn FunctionDescriptor>));
                eq::register(&mut c);
                c.functions.insert(&(eq() as Arc<dyn FunctionDescriptor>));
                ne::register(&mut c);
                c.functions.insert(&(ne() as Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

macro_rules! floating {
    ($mel_type_name:expr, $mel_type:ident, $mel_value_type:ident, $recv_func:ident) => {
        mod $mel_value_type {
            use crate::core::prelude::*;

            impl_Comparison!("GreaterThan", $mel_type_name, $mel_type, gt, $recv_func, send_bool);
            impl_comp_function!("gt", $mel_type_name, $mel_type, $mel_value_type, gt);
            impl_Comparison!("LowerThan", $mel_type_name, $mel_type, lt, $recv_func, send_bool);
            impl_comp_function!("lt", $mel_type_name, $mel_type, $mel_value_type, lt);

            pub fn register(mut c: &mut CollectionPool) {

                gt::register(&mut c);
                c.functions.insert(&(gt() as Arc<dyn FunctionDescriptor>));
                lt::register(&mut c);
                c.functions.insert(&(lt() as Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

integer!("u8", U8, u8, recv_one_u8);
integer!("u16", U16, u16, recv_one_u16);
integer!("u32", U32, u32, recv_one_u32);
integer!("u64", U64, u64, recv_one_u64);
integer!("u128", U128, u128, recv_one_u128);
integer!("i8", I8, i8, recv_one_i8);
integer!("i16", I16, i16, recv_one_i16);
integer!("i32", I32, i32, recv_one_i32);
integer!("i64", I64, i64, recv_one_i64);
integer!("i128", I128, i128, recv_one_i128);

floating!("f32", F32, f32, recv_one_f32);
floating!("f64", F64, f64, recv_one_f64);

pub fn register(mut c: &mut CollectionPool) {

    u8::register(&mut c);
    u16::register(&mut c);
    u32::register(&mut c);
    u64::register(&mut c);
    u128::register(&mut c);
    i8::register(&mut c);
    i16::register(&mut c);
    i32::register(&mut c);
    i64::register(&mut c);
    i128::register(&mut c);

    f32::register(&mut c);
    f64::register(&mut c);
}
