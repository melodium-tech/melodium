
use crate::core::prelude::*;

macro_rules! impl_StaticAddScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        use crate::core::prelude::*;
        treatment!(static_add,
            core_identifier!("arithmetic","scalar";&format!("StaticAdd{}", $mel_name)),
            format!(r"Add a static value to `{}`.

            Every number passed through the stream get `add` added.", stringify!($mel_value_type)),
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

macro_rules! impl_AddScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(add,
            core_identifier!("arithmetic","scalar";&format!("Add{}", $mel_name)),
            format!(r"Add values from two streams of `{}`.

            Values passed through a & b are added and send in sum.", stringify!($mel_value_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("a",Scalar,$mel_type,Stream),
                input!("b",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("sum",Scalar,$mel_type,Stream)
            ],
            host {
                let input_a = host.get_input("a");
                let input_b = host.get_input("b");
                let sum = host.get_output("sum");
            
                while let (Ok(a), Ok(b)) = futures::join!(input_a.$recv_func(), input_b.$recv_func()) {

                    ok_or_break!(sum.$send_func(a + b).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_add_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn add_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn add(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() + params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|add"),
                "Adds a & b".to_string(),
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
            format!(r"Substract a static value to `{}`.

            Every number passed through the stream get `sub` substracted.", stringify!($mel_value_type)),
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

macro_rules! impl_SubScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(sub,
            core_identifier!("arithmetic","scalar";&format!("Sub{}", $mel_name)),
            format!(r"Substract values from two streams of `{}`.

            Every `a` number passed through the stream get `b` substracted.", stringify!($mel_value_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("a",Scalar,$mel_type,Stream),
                input!("b",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("difference",Scalar,$mel_type,Stream)
            ],
            host {
                let input_a = host.get_input("a");
                let input_b = host.get_input("b");
                let difference = host.get_output("difference");
            
                while let (Ok(a), Ok(b)) = futures::join!(input_a.$recv_func(), input_b.$recv_func()) {

                    ok_or_break!(difference.$send_func(a - b).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_sub_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn sub_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn sub(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() - params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|sub"),
                "Substract `b` from `a`".to_string(),
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
            format!(r"Multiply `{}` by static value.

            Every number passed through the stream is multiplied by `factor`.", stringify!($mel_value_type)),
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

macro_rules! impl_MultScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(mult,
            core_identifier!("arithmetic","scalar";&format!("Mult{}", $mel_name)),
            format!(r"Multiply values from two streams of `{}`.

            Every `a` number passed through the stream is multiplied by `b`.", stringify!($mel_value_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("a",Scalar,$mel_type,Stream),
                input!("b",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("product",Scalar,$mel_type,Stream)
            ],
            host {
                let input_a = host.get_input("a");
                let input_b = host.get_input("b");
                let product = host.get_output("product");
            
                while let (Ok(a), Ok(b)) = futures::join!(input_a.$recv_func(), input_b.$recv_func()) {

                    ok_or_break!(product.$send_func(a * b).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_mult_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn mult_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn mult(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() * params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|mult"),
                "Multiply `a` by `b`".to_string(),
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
            format!(r"Divide a stream of `{}` by a static value.

            Every number passed through the stream is divided by `divisor`.", stringify!($mel_value_type)),
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

macro_rules! impl_DivScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(div,
            core_identifier!("arithmetic","scalar";&format!("Div{}", $mel_name)),
            format!(r"Divide values from two streams of `{}`.

            Every `a` number passed through the stream is divided by `b`.", stringify!($mel_value_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("a",Scalar,$mel_type,Stream),
                input!("b",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("quotient",Scalar,$mel_type,Stream)
            ],
            host {
                let input_a = host.get_input("a");
                let input_b = host.get_input("b");
                let quotient = host.get_output("quotient");
            
                while let (Ok(a), Ok(b)) = futures::join!(input_a.$recv_func(), input_b.$recv_func()) {

                    ok_or_break!(quotient.$send_func(a / b).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_div_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn div_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn div(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() / params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|div"),
                "Divide `dividend` by `divisor`".to_string(),
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
            format!(r"Give the remainder of a stream of `{}` divided by a static value.

            Every number passed through the stream is divided by `divisor` and the remainder is outputted.", stringify!($mel_value_type)),
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

macro_rules! impl_RemScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(rem,
            core_identifier!("arithmetic","scalar";&format!("Rem{}", $mel_name)),
            format!(r"Give the remainder of the division from two streams of `{}`.

            Every `a` number passed through the stream is divided by `b` and the remainder is outputted.", stringify!($mel_value_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("a",Scalar,$mel_type,Stream),
                input!("b",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("remainder",Scalar,$mel_type,Stream)
            ],
            host {
                let input_a = host.get_input("a");
                let input_b = host.get_input("b");
                let remainder = host.get_output("remainder");
            
                while let (Ok(a), Ok(b)) = futures::join!(input_a.$recv_func(), input_b.$recv_func()) {

                    ok_or_break!(remainder.$send_func(a % b).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_rem_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn rem_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn rem(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type() % params[1].clone().$mel_value_type())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|rem"),
                "Give the remainder of `dividend` divided by `divisor`.".to_string(),
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
            format!(r"Elevates `{}` to the power of a static value.

            Every number passed through the stream get elevated to the power of `exponent`.", stringify!($mel_value_type)),
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

macro_rules! impl_PowScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(pow,
            core_identifier!("arithmetic","scalar";&format!("Pow{}", $mel_name)),
            format!(r"Elevates values from a stream of `{}` to the power of another one.

            Values passed through `base` are elevated to the power of `exponent`.", stringify!($mel_value_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("base",Scalar,$mel_type,Stream),
                input!("exponent",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("power",Scalar,$mel_type,Stream)
            ],
            host {
                let input_base = host.get_input("base");
                let input_exponent = host.get_input("exponent");
                let power = host.get_output("power");
            
                while let (Ok(a), Ok(b)) = futures::join!(input_base.$recv_func(), input_exponent.$recv_func()) {

                    ok_or_break!(power.$send_func(a.pow(b as u32)).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_pow_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn pow_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn pow(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().pow(params[1].clone().$mel_value_type() as u32))
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|pow"),
                "Elevates `base` from `exponent`".to_string(),
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
            format!(r"Elevates `{}` to the power of a static value.

            Every number passed through the stream get elevated to the power of `exponent`.", stringify!($mel_value_type)),
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

macro_rules! impl_PowfScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(pow,
            core_identifier!("arithmetic","scalar";&format!("Pow{}", $mel_name)),
            format!(r"Elevates values from a stream of `{}` to the power of another one.

            Values passed through `base` are elevated to the power of `exponent`.", stringify!($mel_value_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("base",Scalar,$mel_type,Stream),
                input!("exponent",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("power",Scalar,$mel_type,Stream)
            ],
            host {
                let input_base = host.get_input("base");
                let input_exponent = host.get_input("exponent");
                let power = host.get_output("power");
            
                while let (Ok(a), Ok(b)) = futures::join!(input_base.$recv_func(), input_exponent.$recv_func()) {

                    ok_or_break!(power.$send_func(a.powf(b)).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_powf_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn pow_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn pow(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().powf(params[1].clone().$mel_value_type()))
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|pow"),
                "Elevates `base` from `exponent`".to_string(),
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
            format!(r"Get the absolute values from a stream of `{}`.", stringify!($mel_value_type)),
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
        fn abs_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn abs(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().abs())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|abs"),
                "Get the absolute value".to_string(),
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
            format!(r"Computes the square roots from a stream of `{}`.", stringify!($mel_value_type)),
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
        fn sqrt_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn sqrt(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().sqrt())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|sqrt"),
                "Computes square root of value".to_string(),
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
            format!(r"Computes the cube roots from a stream of `{}`.", stringify!($mel_value_type)),
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
        fn cbrt_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn cbrt(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().cbrt())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|cbrt"),
                "Computes cube root of value".to_string(),
                parameters![
                    parameter!("value", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                cbrt
            )
        }
    };
}

macro_rules! impl_LnScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(ln,
            core_identifier!("arithmetic","scalar";&format!("Ln{}", $mel_name)),
            format!(r"Computes the natural logarithms of a stream of `{}`.", stringify!($mel_value_type)),
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

                    ok_or_break!(output.$send_func(values.iter().map(|v| v.ln()).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_ln_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn ln_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn ln(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().ln())
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|ln"),
                "Computes natural logarithm of value".to_string(),
                parameters![
                    parameter!("value", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                ln
            )
        }
    };
}

macro_rules! impl_StaticLogScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(static_log,
            core_identifier!("arithmetic","scalar";&format!("StaticLog{}", $mel_name)),
            format!(r"Computes the logarithms from a stream of `{}` with respect to a static base.", stringify!($mel_value_type)),
            models![],
            treatment_sources![],
            parameters![
                parameter!("base",Var,Scalar,$mel_type,Some(Value::$mel_type(<$rust_type>::default())))
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

                let base = host.get_parameter("base").$mel_value_type();
            
                while let Ok(values) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(values.iter().map(|v| v.log(base)).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_LogScalar {
    ($mel_name:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!(log,
            core_identifier!("arithmetic","scalar";&format!("Log{}", $mel_name)),
            format!(r"Computes logarithms from a stream of `{}` with the base of another one.", stringify!($mel_value_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream),
                input!("base",Scalar,$mel_type,Stream)
            ],
            outputs![
                output!("log",Scalar,$mel_type,Stream)
            ],
            host {
                let input_base = host.get_input("base");
                let input_value = host.get_input("value");
                let log = host.get_output("log");
            
                while let (Ok(a), Ok(b)) = futures::join!(input_base.$recv_func(), input_value.$recv_func()) {

                    ok_or_break!(log.$send_func(b.log(a)).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

macro_rules! impl_log_function {
    ($mel_name_low:expr, $mel_type:ident, $mel_value_type:ident) => {
        fn log_function() -> std::sync::Arc<CoreFunctionDescriptor> {

            fn log(params: Vec<Value>) -> Value {
                Value::$mel_type(params[0].clone().$mel_value_type().log(params[1].clone().$mel_value_type()))
            }
        
            CoreFunctionDescriptor::new(
                core_identifier!("arithmetic","scalar",$mel_name_low;"|log"),
                "Computes logarithm of value  with the base".to_string(),
                parameters![
                    parameter!("value", Scalar, $mel_type, None),
                    parameter!("base", Scalar, $mel_type, None)
                ],
                datatype!(Scalar, $mel_type),
                log
            )
        }
    };
}

macro_rules! impl_CommonArithm {
    ($mod:ident, $mel_name:expr, $mel_name_low:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $recv_one_func:ident, $send_func:ident, $send_one_func:ident) => {
        pub mod $mod {
            impl_StaticAddScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_AddScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_one_func, $send_one_func);
            impl_add_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_StaticSubScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_SubScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_one_func, $send_one_func);
            impl_sub_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_StaticMultScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_MultScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_one_func, $send_one_func);
            impl_mult_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_StaticDivScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_DivScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_one_func, $send_one_func);
            impl_div_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_StaticRemScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_RemScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_one_func, $send_one_func);
            impl_rem_function!($mel_name_low, $mel_type, $mel_value_type);

            pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {

                static_add::register(&mut c);
                add::register(&mut c);
                c.functions.insert(&(add_function() as std::sync::Arc<dyn FunctionDescriptor>));

                static_sub::register(&mut c);
                sub::register(&mut c);
                c.functions.insert(&(sub_function() as std::sync::Arc<dyn FunctionDescriptor>));

                static_mult::register(&mut c);
                mult::register(&mut c);
                c.functions.insert(&(mult_function() as std::sync::Arc<dyn FunctionDescriptor>));

                static_div::register(&mut c);
                div::register(&mut c);
                c.functions.insert(&(div_function() as std::sync::Arc<dyn FunctionDescriptor>));

                static_rem::register(&mut c);
                rem::register(&mut c);
                c.functions.insert(&(rem_function() as std::sync::Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

macro_rules! impl_IntegerArithm {
    ($mod:ident, $mel_name:expr, $mel_name_low:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $recv_one_func:ident, $send_func:ident, $send_one_func:ident) => {
        pub mod $mod {

            impl_StaticPowScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_PowScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_one_func, $send_one_func);
            impl_pow_function!($mel_name_low, $mel_type, $mel_value_type);
    
            pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {
    
                static_pow::register(&mut c);
                pow::register(&mut c);
                c.functions.insert(&(pow_function() as std::sync::Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

macro_rules! impl_SignedArithm {
    ($mod:ident, $mel_name:expr, $mel_name_low:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $recv_one_func:ident, $send_func:ident, $send_one_func:ident) => {
        pub mod $mod {

            impl_AbsScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_abs_function!($mel_name_low, $mel_type, $mel_value_type);
    
            pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {
    
                absolute::register(&mut c);
                c.functions.insert(&(abs_function() as std::sync::Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

macro_rules! impl_FloatingArithm {
    ($mod:ident, $mel_name:expr, $mel_name_low:expr, $mel_type:ident, $rust_type:ty, $mel_value_type:ident, $recv_func:ident, $recv_one_func:ident, $send_func:ident, $send_one_func:ident) => {
        pub mod $mod {

            impl_StaticPowfScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_PowfScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_one_func, $send_one_func);
            impl_powf_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_SqrtScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_sqrt_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_CbrtScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_cbrt_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_LnScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_ln_function!($mel_name_low, $mel_type, $mel_value_type);

            impl_StaticLogScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_func, $send_func);
            impl_LogScalar!($mel_name, $mel_type, $rust_type, $mel_value_type, $recv_one_func, $send_one_func);
            impl_log_function!($mel_name_low, $mel_type, $mel_value_type);

            pub fn register(mut c: &mut crate::logic::collection_pool::CollectionPool) {

                static_pow::register(&mut c);
                pow::register(&mut c);
                c.functions.insert(&(pow_function() as std::sync::Arc<dyn FunctionDescriptor>));

                sqrt::register(&mut c);
                c.functions.insert(&(sqrt_function() as std::sync::Arc<dyn FunctionDescriptor>));

                cbrt::register(&mut c);
                c.functions.insert(&(cbrt_function() as std::sync::Arc<dyn FunctionDescriptor>));

                ln::register(&mut c);
                c.functions.insert(&(ln_function() as std::sync::Arc<dyn FunctionDescriptor>));

                static_log::register(&mut c);
                log::register(&mut c);
                c.functions.insert(&(log_function() as std::sync::Arc<dyn FunctionDescriptor>));
            }
        }
    };
}

mod common {
    use crate::core::prelude::*;

    impl_CommonArithm!(u8,    "U8",   "u8",   U8,     u8,     u8,     recv_u8,    recv_one_u8,   send_multiple_u8,   send_u8    );
    impl_CommonArithm!(u16,   "U16",  "u16",  U16,    u16,    u16,    recv_u16,   recv_one_u16,  send_multiple_u16,  send_u16   );
    impl_CommonArithm!(u32,   "U32",  "u32",  U32,    u32,    u32,    recv_u32,   recv_one_u32,  send_multiple_u32,  send_u32   );
    impl_CommonArithm!(u64,   "U64",  "u64",  U64,    u64,    u64,    recv_u64,   recv_one_u64,  send_multiple_u64,  send_u64   );
    impl_CommonArithm!(u128,  "U128", "u128", U128,   u128,   u128,   recv_u128,  recv_one_u128, send_multiple_u128, send_u128  );
    impl_CommonArithm!(i8,    "I8",   "i8",   I8,     i8,     i8,     recv_i8,    recv_one_i8,   send_multiple_i8,   send_i8    );
    impl_CommonArithm!(i16,   "I16",  "i16",  I16,    i16,    i16,    recv_i16,   recv_one_i16,  send_multiple_i16,  send_i16   );
    impl_CommonArithm!(i32,   "I32",  "i32",  I32,    i32,    i32,    recv_i32,   recv_one_i32,  send_multiple_i32,  send_i32   );
    impl_CommonArithm!(i64,   "I64",  "i64",  I64,    i64,    i64,    recv_i64,   recv_one_i64,  send_multiple_i64,  send_i64   );
    impl_CommonArithm!(i128,  "I128", "i128", I128,   i128,   i128,   recv_i128,  recv_one_i128, send_multiple_i128, send_i128  );
    impl_CommonArithm!(f32,   "F32",  "f32",  F32,    f32,    f32,    recv_f32,   recv_one_f32,  send_multiple_f32,  send_f32   );
    impl_CommonArithm!(f64,   "F64",  "f64",  F64,    f64,    f64,    recv_f64,   recv_one_f64,  send_multiple_f64,  send_f64   );

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

    impl_IntegerArithm!(u8,    "U8",   "u8",   U8,     u8,     u8,     recv_u8,    recv_one_u8,   send_multiple_u8,   send_u8    );
    impl_IntegerArithm!(u16,   "U16",  "u16",  U16,    u16,    u16,    recv_u16,   recv_one_u16,  send_multiple_u16,  send_u16   );
    impl_IntegerArithm!(u32,   "U32",  "u32",  U32,    u32,    u32,    recv_u32,   recv_one_u32,  send_multiple_u32,  send_u32   );
    impl_IntegerArithm!(u64,   "U64",  "u64",  U64,    u64,    u64,    recv_u64,   recv_one_u64,  send_multiple_u64,  send_u64   );
    impl_IntegerArithm!(u128,  "U128", "u128", U128,   u128,   u128,   recv_u128,  recv_one_u128, send_multiple_u128, send_u128  );
    impl_IntegerArithm!(i8,    "I8",   "i8",   I8,     i8,     i8,     recv_i8,    recv_one_i8,   send_multiple_i8,   send_i8    );
    impl_IntegerArithm!(i16,   "I16",  "i16",  I16,    i16,    i16,    recv_i16,   recv_one_i16,  send_multiple_i16,  send_i16   );
    impl_IntegerArithm!(i32,   "I32",  "i32",  I32,    i32,    i32,    recv_i32,   recv_one_i32,  send_multiple_i32,  send_i32   );
    impl_IntegerArithm!(i64,   "I64",  "i64",  I64,    i64,    i64,    recv_i64,   recv_one_i64,  send_multiple_i64,  send_i64   );
    impl_IntegerArithm!(i128,  "I128", "i128", I128,   i128,   i128,   recv_i128,  recv_one_i128, send_multiple_i128, send_i128  );

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

    impl_SignedArithm!(i8,    "I8",   "i8",   I8,     i8,     i8,     recv_i8,    recv_one_i8,   send_multiple_i8,   send_i8    );
    impl_SignedArithm!(i16,   "I16",  "i16",  I16,    i16,    i16,    recv_i16,   recv_one_i16,  send_multiple_i16,  send_i16   );
    impl_SignedArithm!(i32,   "I32",  "i32",  I32,    i32,    i32,    recv_i32,   recv_one_i32,  send_multiple_i32,  send_i32   );
    impl_SignedArithm!(i64,   "I64",  "i64",  I64,    i64,    i64,    recv_i64,   recv_one_i64,  send_multiple_i64,  send_i64   );
    impl_SignedArithm!(i128,  "I128", "i128", I128,   i128,   i128,   recv_i128,  recv_one_i128, send_multiple_i128, send_i128  );
    impl_SignedArithm!(f32,   "F32",  "f32",  F32,    f32,    f32,    recv_f32,   recv_one_f32,  send_multiple_f32,  send_f32   );
    impl_SignedArithm!(f64,   "F64",  "f64",  F64,    f64,    f64,    recv_f64,   recv_one_f64,  send_multiple_f64,  send_f64   );

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

    impl_FloatingArithm!(f32,   "F32",  "f32",  F32,    f32,    f32,    recv_f32,   recv_one_f32,  send_multiple_f32,  send_f32   );
    impl_FloatingArithm!(f64,   "F64",  "f64",  F64,    f64,    f64,    recv_f64,   recv_one_f64,  send_multiple_f64,  send_f64   );

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

