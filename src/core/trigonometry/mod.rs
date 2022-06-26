

macro_rules! impl_Trigo {
    ($mel_name:expr, $mel_type_up:ident, $mel_type_name:expr, $func:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($func,
            core_identifier!("trigonometry","scalar",$mel_type_name;$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type_up,Stream)
            ],
            outputs![
                output!("value",Scalar,$mel_type_up,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v.$func()).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_trigo_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_type_name:expr, $mel_value_type:ident, $func:ident) => {
        fn $func() -> Arc<CoreFunctionDescriptor> {

            fn func(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().$func())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("trigonometry","scalar",$mel_type_name;&format!("|{}", $mel_name_low)),
                parameters![
                    parameter!("value", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                func
            )
        }
    };
}

mod f32 {

    use crate::core::prelude::*;

    impl_Trigo!("Sin", F32, "f32", sin, recv_f32, send_multiple_f32);
    impl_Trigo!("Cos", F32, "f32", cos, recv_f32, send_multiple_f32);
    impl_Trigo!("Tan", F32, "f32", tan, recv_f32, send_multiple_f32);
    impl_Trigo!("Asin", F32, "f32", asin, recv_f32, send_multiple_f32);
    impl_Trigo!("Acos", F32, "f32", acos, recv_f32, send_multiple_f32);
    impl_Trigo!("Atan", F32, "f32", atan, recv_f32, send_multiple_f32);
    impl_Trigo!("Sinh", F32, "f32", sinh, recv_f32, send_multiple_f32);
    impl_Trigo!("Cosh", F32, "f32", cosh, recv_f32, send_multiple_f32);
    impl_Trigo!("Tanh", F32, "f32", tanh, recv_f32, send_multiple_f32);
    impl_Trigo!("Asinh", F32, "f32", asinh, recv_f32, send_multiple_f32);
    impl_Trigo!("Acosh", F32, "f32", acosh, recv_f32, send_multiple_f32);
    impl_Trigo!("Atanh", F32, "f32", atanh, recv_f32, send_multiple_f32);

    impl_trigo_function!("sin", F32, "f32", f32, sin);
    impl_trigo_function!("cos", F32, "f32", f32, cos);
    impl_trigo_function!("tan", F32, "f32", f32, tan);
    impl_trigo_function!("asin", F32, "f32", f32, asin);
    impl_trigo_function!("acos", F32, "f32", f32, acos);
    impl_trigo_function!("atan", F32, "f32", f32, atan);
    impl_trigo_function!("sinh", F32, "f32", f32, sinh);
    impl_trigo_function!("cosh", F32, "f32", f32, cosh);
    impl_trigo_function!("tanh", F32, "f32", f32, tanh);
    impl_trigo_function!("asinh", F32, "f32", f32, asinh);
    impl_trigo_function!("acosh", F32, "f32", f32, acosh);
    impl_trigo_function!("atanh", F32, "f32", f32, atanh);

    pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {

        sin::register(&mut c);
        c.functions.insert(&(sin() as Arc<dyn FunctionDescriptor>));
        cos::register(&mut c);
        c.functions.insert(&(cos() as Arc<dyn FunctionDescriptor>));
        tan::register(&mut c);
        c.functions.insert(&(tan() as Arc<dyn FunctionDescriptor>));
        asin::register(&mut c);
        c.functions.insert(&(asin() as Arc<dyn FunctionDescriptor>));
        acos::register(&mut c);
        c.functions.insert(&(acos() as Arc<dyn FunctionDescriptor>));
        atan::register(&mut c);
        c.functions.insert(&(atan() as Arc<dyn FunctionDescriptor>));
        sinh::register(&mut c);
        c.functions.insert(&(sinh() as Arc<dyn FunctionDescriptor>));
        cosh::register(&mut c);
        c.functions.insert(&(cosh() as Arc<dyn FunctionDescriptor>));
        tanh::register(&mut c);
        c.functions.insert(&(tanh() as Arc<dyn FunctionDescriptor>));
        asinh::register(&mut c);
        c.functions.insert(&(asinh() as Arc<dyn FunctionDescriptor>));
        acosh::register(&mut c);
        c.functions.insert(&(acosh() as Arc<dyn FunctionDescriptor>));
        atanh::register(&mut c);
        c.functions.insert(&(atanh() as Arc<dyn FunctionDescriptor>));
    }
}

mod f64 {

    use crate::core::prelude::*;

    impl_Trigo!("Sin", F64, "f64", sin, recv_f64, send_multiple_f64);
    impl_Trigo!("Cos", F64, "f64", cos, recv_f64, send_multiple_f64);
    impl_Trigo!("Tan", F64, "f64", tan, recv_f64, send_multiple_f64);
    impl_Trigo!("Asin", F64, "f64", asin, recv_f64, send_multiple_f64);
    impl_Trigo!("Acos", F64, "f64", acos, recv_f64, send_multiple_f64);
    impl_Trigo!("Atan", F64, "f64", atan, recv_f64, send_multiple_f64);
    impl_Trigo!("Sinh", F64, "f64", sinh, recv_f64, send_multiple_f64);
    impl_Trigo!("Cosh", F64, "f64", cosh, recv_f64, send_multiple_f64);
    impl_Trigo!("Tanh", F64, "f64", tanh, recv_f64, send_multiple_f64);
    impl_Trigo!("Asinh", F64, "f64", asinh, recv_f64, send_multiple_f64);
    impl_Trigo!("Acosh", F64, "f64", acosh, recv_f64, send_multiple_f64);
    impl_Trigo!("Atanh", F64, "f64", atanh, recv_f64, send_multiple_f64);

    impl_trigo_function!("sin", F64, "f64", f64, sin);
    impl_trigo_function!("cos", F64, "f64", f64, cos);
    impl_trigo_function!("tan", F64, "f64", f64, tan);
    impl_trigo_function!("asin", F64, "f64", f64, asin);
    impl_trigo_function!("acos", F64, "f64", f64, acos);
    impl_trigo_function!("atan", F64, "f64", f64, atan);
    impl_trigo_function!("sinh", F64, "f64", f64, sinh);
    impl_trigo_function!("cosh", F64, "f64", f64, cosh);
    impl_trigo_function!("tanh", F64, "f64", f64, tanh);
    impl_trigo_function!("asinh", F64, "f64", f64, asinh);
    impl_trigo_function!("acosh", F64, "f64", f64, acosh);
    impl_trigo_function!("atanh", F64, "f64", f64, atanh);

    pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {

        sin::register(&mut c);
        c.functions.insert(&(sin() as Arc<dyn FunctionDescriptor>));
        cos::register(&mut c);
        c.functions.insert(&(cos() as Arc<dyn FunctionDescriptor>));
        tan::register(&mut c);
        c.functions.insert(&(tan() as Arc<dyn FunctionDescriptor>));
        asin::register(&mut c);
        c.functions.insert(&(asin() as Arc<dyn FunctionDescriptor>));
        acos::register(&mut c);
        c.functions.insert(&(acos() as Arc<dyn FunctionDescriptor>));
        atan::register(&mut c);
        c.functions.insert(&(atan() as Arc<dyn FunctionDescriptor>));
        sinh::register(&mut c);
        c.functions.insert(&(sinh() as Arc<dyn FunctionDescriptor>));
        cosh::register(&mut c);
        c.functions.insert(&(cosh() as Arc<dyn FunctionDescriptor>));
        tanh::register(&mut c);
        c.functions.insert(&(tanh() as Arc<dyn FunctionDescriptor>));
        asinh::register(&mut c);
        c.functions.insert(&(asinh() as Arc<dyn FunctionDescriptor>));
        acosh::register(&mut c);
        c.functions.insert(&(acosh() as Arc<dyn FunctionDescriptor>));
        atanh::register(&mut c);
        c.functions.insert(&(atanh() as Arc<dyn FunctionDescriptor>));
    }
}

pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {

    f32::register(&mut c);
    f64::register(&mut c);
}
