

macro_rules! impl_Trigo {
    ($mel_name:expr, $doc:expr, $mel_type_up:ident, $mel_type_name:expr, $func:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($func,
            core_identifier!("trigonometry","scalar",$mel_type_name;$mel_name),
            $doc.to_string(),
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
    ($mel_name_low:expr, $doc:expr, $mel_type:ident, $mel_type_name:expr, $mel_value_type:ident, $func:ident) => {
        fn $func() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn func(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().$func())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("trigonometry","scalar",$mel_type_name;&format!("|{}", $mel_name_low)),
                $doc.to_string(),
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

    use std::sync::Arc;
    use crate::core::prelude::*;

    impl_Trigo!("Sin", "Computes sine (in radians) of a stream of `f32`.", F32, "f32", sin, recv_f32, send_multiple_f32);
    impl_Trigo!("Cos", "Computes cosine (in radians) of a stream of `f32`.", F32, "f32", cos, recv_f32, send_multiple_f32);
    impl_Trigo!("Tan", "Computes tangent (in radians) of a stream of `f32`.", F32, "f32", tan, recv_f32, send_multiple_f32);
    impl_Trigo!("Asin", r"Computes arcsine (in radians) of a stream of `f32`.

    Gives values in the range [0, pi] or not-a-number if outside range [-1, 1].", F32, "f32", asin, recv_f32, send_multiple_f32);
    impl_Trigo!("Acos", r"Computes arccosine (in radians) of a stream of `f32`.

    Gives values in the range [0, pi] or not-a-number if outside range [-1, 1].", F32, "f32", acos, recv_f32, send_multiple_f32);
    impl_Trigo!("Atan", r"Computes arctangent (in radians) of a stream of `f32`.

    Gives values in the range [-pi/2, pi/2].", F32, "f32", atan, recv_f32, send_multiple_f32);
    impl_Trigo!("Sinh", "Computes hyberbolic sine of a stream of `f32`.",F32, "f32", sinh, recv_f32, send_multiple_f32);
    impl_Trigo!("Cosh", "Computes hyberbolic cosine of a stream of `f32`.", F32, "f32", cosh, recv_f32, send_multiple_f32);
    impl_Trigo!("Tanh", "Computes hyberbolic tangent of a stream of `f32`.", F32, "f32", tanh, recv_f32, send_multiple_f32);
    impl_Trigo!("Asinh", "Computes inverse hyperbolic sine of a stream of `f32`.", F32, "f32", asinh, recv_f32, send_multiple_f32);
    impl_Trigo!("Acosh", "Computes inverse hyperbolic cosine of a stream of `f32`.", F32, "f32", acosh, recv_f32, send_multiple_f32);
    impl_Trigo!("Atanh", "Computes inverse hyperbolic tangent of a stream of `f32`.", F32, "f32", atanh, recv_f32, send_multiple_f32);

    impl_trigo_function!("sin", "Computes sine (in radians).", F32, "f32", f32, sin);
    impl_trigo_function!("cos", "Computes cosine (in radians).", F32, "f32", f32, cos);
    impl_trigo_function!("tan", "Computes tangent (in radians).", F32, "f32", f32, tan);
    impl_trigo_function!("asin", "Computes arcsine (in radians).", F32, "f32", f32, asin);
    impl_trigo_function!("acos", "Computes arccosine (in radians).", F32, "f32", f32, acos);
    impl_trigo_function!("atan", "Computes arctangent (in radians).", F32, "f32", f32, atan);
    impl_trigo_function!("sinh", "Computes hyberbolic sine.", F32, "f32", f32, sinh);
    impl_trigo_function!("cosh", "Computes hyperbolic cosine.", F32, "f32", f32, cosh);
    impl_trigo_function!("tanh", "Computes hyperbolic tangent.", F32, "f32", f32, tanh);
    impl_trigo_function!("asinh", "Computes inverse hyperbolic sine.", F32, "f32", f32, asinh);
    impl_trigo_function!("acosh", "Computes inverse hyperbolic cosine.", F32, "f32", f32, acosh);
    impl_trigo_function!("atanh", "Computes inverse hyperbolic tangent.", F32, "f32", f32, atanh);

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

    use std::sync::Arc;
    use crate::core::prelude::*;

    impl_Trigo!("Sin", "Computes sine (in radians) of a stream of `f64`.", F64, "f64", sin, recv_f64, send_multiple_f64);
    impl_Trigo!("Cos", "Computes cosine (in radians) of a stream of `f64`.", F64, "f64", cos, recv_f64, send_multiple_f64);
    impl_Trigo!("Tan", "Computes tangent (in radians) of a stream of `f64`.", F64, "f64", tan, recv_f64, send_multiple_f64);
    impl_Trigo!("Asin", r"Computes arcsine (in radians) of a stream of `f64`.

    Gives values in the range [0, pi] or not-a-number if outside range [-1, 1].", F64, "f64", asin, recv_f64, send_multiple_f64);
    impl_Trigo!("Acos", r"Computes arccosine (in radians) of a stream of `f64`.

    Gives values in the range [0, pi] or not-a-number if outside range [-1, 1].", F64, "f64", acos, recv_f64, send_multiple_f64);
    impl_Trigo!("Atan", r"Computes arctangent (in radians) of a stream of `f64`.

    Gives values in the range [-pi/2, pi/2].", F64, "f64", atan, recv_f64, send_multiple_f64);
    impl_Trigo!("Sinh", "Computes hyberbolic sine of a stream of `f64`.",F64, "f64", sinh, recv_f64, send_multiple_f64);
    impl_Trigo!("Cosh", "Computes hyberbolic cosine of a stream of `f64`.", F64, "f64", cosh, recv_f64, send_multiple_f64);
    impl_Trigo!("Tanh", "Computes hyberbolic tangent of a stream of `f64`.", F64, "f64", tanh, recv_f64, send_multiple_f64);
    impl_Trigo!("Asinh", "Computes inverse hyperbolic sine of a stream of `f64`.", F64, "f64", asinh, recv_f64, send_multiple_f64);
    impl_Trigo!("Acosh", "Computes inverse hyperbolic cosine of a stream of `f64`.", F64, "f64", acosh, recv_f64, send_multiple_f64);
    impl_Trigo!("Atanh", "Computes inverse hyperbolic tangent of a stream of `f64`.", F64, "f64", atanh, recv_f64, send_multiple_f64);

    impl_trigo_function!("sin", "Computes sine (in radians).", F64, "f64", f64, sin);
    impl_trigo_function!("cos", "Computes cosine (in radians).", F64, "f64", f64, cos);
    impl_trigo_function!("tan", "Computes tangent (in radians).", F64, "f64", f64, tan);
    impl_trigo_function!("asin", "Computes arcsine (in radians).", F64, "f64", f64, asin);
    impl_trigo_function!("acos", "Computes arccosine (in radians).", F64, "f64", f64, acos);
    impl_trigo_function!("atan", "Computes arctangent (in radians).", F64, "f64", f64, atan);
    impl_trigo_function!("sinh", "Computes hyberbolic sine.", F64, "f64", f64, sinh);
    impl_trigo_function!("cosh", "Computes hyperbolic cosine.", F64, "f64", f64, cosh);
    impl_trigo_function!("tanh", "Computes hyperbolic tangent.", F64, "f64", f64, tanh);
    impl_trigo_function!("asinh", "Computes inverse hyperbolic sine.", F64, "f64", f64, asinh);
    impl_trigo_function!("acosh", "Computes inverse hyperbolic cosine.", F64, "f64", f64, acosh);
    impl_trigo_function!("atanh", "Computes inverse hyperbolic tangent.", F64, "f64", f64, atanh);

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
