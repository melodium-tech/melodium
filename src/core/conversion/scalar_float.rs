
use crate::core::prelude::*;

macro_rules! impl_ScalarFloat {
    ($mod:ident, $mel_name:expr, $input_mel_type:ident, $recv_func:ident, $input_rust_type:ty, $output_mel_type:ident, $output_rust_type:ty, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("conversion","scalar";$mel_name),
            format!(r"Convert stream of `{in}` into `{out}`.

            Every `{in}` is fitted into the closest `{out}`.
            Positive and negative infinity are conserved, as well as not-a-number state.
            If overflowing, infinity of the same sign is used.", in = stringify!($input_rust_type), out = stringify!($output_rust_type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$input_mel_type,Stream)
            ],
            outputs![
                output!("value",Scalar,$output_mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let output = host.get_output("value");
            
                while let Ok(numbers) = input.$recv_func().await {

                    ok_or_break!(output.$send_func(numbers.iter().map(|n| *n as $output_rust_type).collect()).await);
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_ScalarFloat!(f32_to_f64, "ScalarF32ToF64", F32, recv_f32, f32, F64, f64, send_multiple_f64);
impl_ScalarFloat!(f64_to_f32, "ScalarF64ToF32", F64, recv_f64, f64, F32, f32, send_multiple_f32);

pub fn register(mut c: &mut CollectionPool) {

    f32_to_f64::register(&mut c);
    f64_to_f32::register(&mut c);

}
