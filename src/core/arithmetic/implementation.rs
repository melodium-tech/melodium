
use crate::core::prelude::*;

macro_rules! impl_StaticAddScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        use crate::core::prelude::*;
        treatment!(static_add,
            core_identifier!("arithmetic","scalar";&format!("StaticAdd{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("add",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
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

macro_rules! impl_add_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn add_function() -> Arc<CoreFunctionDescriptor> {

            fn add(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() + params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|add_{}", $mel_name_low)),
                parameters![
                    parameter!("a", Scalar, $mel_type, None),
                    parameter!("b", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                add
            )
        }
    };
}

macro_rules! impl_StaticSubScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(static_sub,
            core_identifier!("arithmetic","scalar";&format!("StaticSub{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("sub",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
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

                let sub = host.get_parameter("sub").$mel_value_type();
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v - sub).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_sub_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn sub_function() -> Arc<CoreFunctionDescriptor> {

            fn sub(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() - params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|sub_{}", $mel_name_low)),
                parameters![
                    parameter!("a", Scalar, $mel_type, None),
                    parameter!("b", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                sub
            )
        }
    };
}

macro_rules! impl_StaticMultScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(static_mult,
            core_identifier!("arithmetic","scalar";&format!("StaticMult{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("factor",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
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

                let factor = host.get_parameter("factor").$mel_value_type();
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v * factor).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_mult_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn mult_function() -> Arc<CoreFunctionDescriptor> {

            fn mult(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() * params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|mult_{}", $mel_name_low)),
                parameters![
                    parameter!("a", Scalar, $mel_type, None),
                    parameter!("b", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                mult
            )
        }
    };
}

macro_rules! impl_StaticDivScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(static_div,
            core_identifier!("arithmetic","scalar";&format!("StaticDiv{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("divisor",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
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

                let divisor = host.get_parameter("divisor").$mel_value_type();
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v / divisor).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_div_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn div_function() -> Arc<CoreFunctionDescriptor> {

            fn div(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() / params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|div_{}", $mel_name_low)),
                parameters![
                    parameter!("dividend", Scalar, $mel_type, None),
                    parameter!("divisor", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                div
            )
        }
    };
}

macro_rules! impl_StaticRemScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(static_rem,
            core_identifier!("arithmetic","scalar";&format!("StaticRem{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("divisor",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
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

                let divisor = host.get_parameter("divisor").$mel_value_type();
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v % divisor).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_rem_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn rem_function() -> Arc<CoreFunctionDescriptor> {

            fn rem(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() % params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|rem_{}", $mel_name_low)),
                parameters![
                    parameter!("dividend", Scalar, $mel_type, None),
                    parameter!("divisor", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                rem
            )
        }
    };
}

macro_rules! impl_StaticPowScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        use crate::core::prelude::*;
        treatment!(static_pow,
            core_identifier!("arithmetic","scalar";&format!("StaticPow{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("exponent",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
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

                let exponent = host.get_parameter("exponent").$mel_value_type();
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v.pow(exponent as u32)).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_pow_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn pow_function() -> Arc<CoreFunctionDescriptor> {

            fn pow(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().pow(params[1].clone().$mel_value_type() as u32))
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|pow_{}", $mel_name_low)),
                parameters![
                    parameter!("base", Scalar, $mel_type, None),
                    parameter!("exponent", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                pow
            )
        }
    };
}

macro_rules! impl_StaticPowfScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        use crate::core::prelude::*;
        treatment!(static_pow,
            core_identifier!("arithmetic","scalar";&format!("StaticPow{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("exponent",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
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

                let exponent = host.get_parameter("exponent").$mel_value_type();
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v.powf(exponent)).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_powf_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn pow_function() -> Arc<CoreFunctionDescriptor> {

            fn pow(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().powf(params[1].clone().$mel_value_type()))
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|pow_{}", $mel_name_low)),
                parameters![
                    parameter!("base", Scalar, $mel_type, None),
                    parameter!("exponent", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                pow
            )
        }
    };
}

macro_rules! impl_AbsScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        use crate::core::prelude::*;
        treatment!(absolute,
            core_identifier!("arithmetic","scalar";&format!("Abs{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v.abs()).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_abs_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn abs_function() -> Arc<CoreFunctionDescriptor> {

            fn abs(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().abs())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|abs_{}", $mel_name_low)),
                parameters![
                    parameter!("value", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                abs
            )
        }
    };
}

macro_rules! impl_SqrtScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(sqrt,
            core_identifier!("arithmetic","scalar";&format!("Sqrt{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v.sqrt()).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_sqrt_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn sqrt_function() -> Arc<CoreFunctionDescriptor> {

            fn sqrt(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().sqrt())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|sqrt_{}", $mel_name_low)),
                parameters![
                    parameter!("value", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                sqrt
            )
        }
    };
}

macro_rules! impl_CbrtScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(cbrt,
            core_identifier!("arithmetic","scalar";&format!("Cbrt{}", $mel_name)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v.cbrt()).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_cbrt_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn cbrt_function() -> Arc<CoreFunctionDescriptor> {

            fn cbrt(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().cbrt())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("func";&format!("|cbrt_{}", $mel_name_low)),
                parameters![
                    parameter!("value", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                cbrt
            )
        }
    };
}

macro_rules! impl_CommonArithm {
    ($mod:ident, $mel_name:expr, $mel_name_low:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        pub mod $mod {
            impl_StaticAddScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_add_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_StaticSubScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_sub_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_StaticMultScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_mult_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_StaticDivScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_div_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_StaticRemScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_rem_function!($mel_name_low, $mel_type, $mel_value_type);

            pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {

                static_add::register(&mut c);
                c.functions.insert(&(add_function() as Arc<dyn FunctionDescriptor>));

                static_sub::register(&mut c);
                c.functions.insert(&(sub_function() as Arc<dyn FunctionDescriptor>));

                static_mult::register(&mut c);
                c.functions.insert(&(mult_function() as Arc<dyn FunctionDescriptor>));

                static_div::register(&mut c);
                c.functions.insert(&(div_function() as Arc<dyn FunctionDescriptor>));

                static_rem::register(&mut c);
                c.functions.insert(&(rem_function() as Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

macro_rules! impl_IntegerArithm {
    ($mod:ident, $mel_name:expr, $mel_name_low:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        pub mod $mod {

            impl_StaticPowScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_pow_function!($mel_name_low, $mel_type, $mel_value_type);
    
            pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {
    
                static_pow::register(&mut c);
                c.functions.insert(&(pow_function() as Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

macro_rules! impl_SignedArithm {
    ($mod:ident, $mel_name:expr, $mel_name_low:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        pub mod $mod {

            impl_AbsScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_abs_function!($mel_name_low, $mel_type, $mel_value_type);
    
            pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {
    
                absolute::register(&mut c);
                c.functions.insert(&(abs_function() as Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

macro_rules! impl_FloatingArithm {
    ($mod:ident, $mel_name:expr, $mel_name_low:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        pub mod $mod {

            impl_StaticPowfScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_powf_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_SqrtScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_sqrt_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_CbrtScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_cbrt_function!($mel_name_low, $mel_type, $mel_value_type);

            pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {

                static_pow::register(&mut c);
                c.functions.insert(&(pow_function() as Arc<dyn FunctionDescriptor>));

                sqrt::register(&mut c);
                c.functions.insert(&(sqrt_function() as Arc<dyn FunctionDescriptor>));

                cbrt::register(&mut c);
                c.functions.insert(&(cbrt_function() as Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

mod common {
    use crate::core::prelude::*;

    impl_CommonArithm!(u8,    "U8",   "u8",   U8,     u8,     u8,     recv_u8,    send_multiple_u8    );
    impl_CommonArithm!(u16,   "U16",  "u16",  U16,    u16,    u16,    recv_u16,   send_multiple_u16   );
    impl_CommonArithm!(u32,   "U32",  "u32",  U32,    u32,    u32,    recv_u32,   send_multiple_u32   );
    impl_CommonArithm!(u64,   "U64",  "u64",  U64,    u64,    u64,    recv_u64,   send_multiple_u64   );
    impl_CommonArithm!(u128,  "U128", "u128", U128,   u128,   u128,   recv_u128,  send_multiple_u128  );
    impl_CommonArithm!(i8,    "I8",   "i8",   I8,     i8,     i8,     recv_i8,    send_multiple_i8    );
    impl_CommonArithm!(i16,   "I16",  "i16",  I16,    i16,    i16,    recv_i16,   send_multiple_i16   );
    impl_CommonArithm!(i32,   "I32",  "i32",  I32,    i32,    i32,    recv_i32,   send_multiple_i32   );
    impl_CommonArithm!(i64,   "I64",  "i64",  I64,    i64,    i64,    recv_i64,   send_multiple_i64   );
    impl_CommonArithm!(i128,  "I128", "i128", I128,   i128,   i128,   recv_i128,  send_multiple_i128  );
    impl_CommonArithm!(f32,   "F32",  "f32",  F32,    f32,    f32,    recv_f32,   send_multiple_f32   );
    impl_CommonArithm!(f64,   "F64",  "f64",  F64,    f64,    f64,    recv_f64,   send_multiple_f64   );

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
}

mod integer {
    use crate::core::prelude::*;

    impl_IntegerArithm!(u8,    "U8",   "u8",   U8,     u8,     u8,     recv_u8,    send_multiple_u8    );
    impl_IntegerArithm!(u16,   "U16",  "u16",  U16,    u16,    u16,    recv_u16,   send_multiple_u16   );
    impl_IntegerArithm!(u32,   "U32",  "u32",  U32,    u32,    u32,    recv_u32,   send_multiple_u32   );
    impl_IntegerArithm!(u64,   "U64",  "u64",  U64,    u64,    u64,    recv_u64,   send_multiple_u64   );
    impl_IntegerArithm!(u128,  "U128", "u128", U128,   u128,   u128,   recv_u128,  send_multiple_u128  );
    impl_IntegerArithm!(i8,    "I8",   "i8",   I8,     i8,     i8,     recv_i8,    send_multiple_i8    );
    impl_IntegerArithm!(i16,   "I16",  "i16",  I16,    i16,    i16,    recv_i16,   send_multiple_i16   );
    impl_IntegerArithm!(i32,   "I32",  "i32",  I32,    i32,    i32,    recv_i32,   send_multiple_i32   );
    impl_IntegerArithm!(i64,   "I64",  "i64",  I64,    i64,    i64,    recv_i64,   send_multiple_i64   );
    impl_IntegerArithm!(i128,  "I128", "i128", I128,   i128,   i128,   recv_i128,  send_multiple_i128  );

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
    }
}

mod signed {
    use crate::core::prelude::*;

    impl_SignedArithm!(i8,    "I8",   "i8",   I8,     i8,     i8,     recv_i8,    send_multiple_i8    );
    impl_SignedArithm!(i16,   "I16",  "i16",  I16,    i16,    i16,    recv_i16,   send_multiple_i16   );
    impl_SignedArithm!(i32,   "I32",  "i32",  I32,    i32,    i32,    recv_i32,   send_multiple_i32   );
    impl_SignedArithm!(i64,   "I64",  "i64",  I64,    i64,    i64,    recv_i64,   send_multiple_i64   );
    impl_SignedArithm!(i128,  "I128", "i128", I128,   i128,   i128,   recv_i128,  send_multiple_i128  );
    impl_SignedArithm!(f32,   "F32",  "f32",  F32,    f32,    f32,    recv_f32,   send_multiple_f32   );
    impl_SignedArithm!(f64,   "F64",  "f64",  F64,    f64,    f64,    recv_f64,   send_multiple_f64   );

    pub fn register(mut c: &mut CollectionPool) {

        i8::register(&mut c);
        i16::register(&mut c);
        i32::register(&mut c);
        i64::register(&mut c);
        i128::register(&mut c);
        f32::register(&mut c);
        f64::register(&mut c);
    }
}

mod floating {
    use crate::core::prelude::*;

    impl_FloatingArithm!(f32,   "F32",  "f32",  F32,    f32,    f32,    recv_f32,   send_multiple_f32   );
    impl_FloatingArithm!(f64,   "F64",  "f64",  F64,    f64,    f64,    recv_f64,   send_multiple_f64   );

    pub fn register(mut c: &mut CollectionPool) {

        f32::register(&mut c);
        f64::register(&mut c);
    }
}


pub fn register(mut c: &mut CollectionPool) {

    common::register(&mut c);
    integer::register(&mut c);
    signed::register(&mut c);
    floating::register(&mut c);
}

