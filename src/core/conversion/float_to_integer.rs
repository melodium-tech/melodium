
use super::super::prelude::*;

macro_rules! impl_ScalarFloatToInteger {
    ($name:ident, $mel_name:expr, $input_rust_type:ty, $input_mel_type:ident, $output_rust_type:ty, $output_mel_type:ident) => {
        pub struct $name {

            world: Arc<World>,
        
            neg_infinity: RwLock<$output_rust_type>,
            pos_infinity: RwLock<$output_rust_type>,
            nan: RwLock<$output_rust_type>,
        
            data_output_transmitters: RwLock<Vec<Transmitter>>,
            data_input_sender: Sender<$input_rust_type>,
            data_input_receiver: Receiver<$input_rust_type>,
        
            auto_reference: RwLock<Weak<Self>>,
        
        }

        impl $name {

            pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {
        
                lazy_static! {
                    static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {
        
                        let rc_descriptor = CoreTreatmentDescriptor::new(
                            core_identifier!("conversion";$mel_name),
                            models![],
                            treatment_sources![],
                            vec![
                                parameter!("neg_infinity",Scalar,$output_mel_type,Some(crate::executive::value::Value::$output_mel_type(<$output_rust_type>::MIN))),
                                parameter!("pos_infinity",Scalar,$output_mel_type,Some(crate::executive::value::Value::$output_mel_type(<$output_rust_type>::MAX))),
                                parameter!("nan",Scalar,$output_mel_type,Some(crate::executive::value::Value::$output_mel_type(<$output_rust_type>::default()))),
                            ],
                            vec![
                                input!("value",Scalar,$input_mel_type,Stream)
                            ],
                            vec![
                                output!("value",Scalar,$output_mel_type,Stream)
                            ],
                            $name::new,
                        );
        
                        rc_descriptor
                    };
                }
        
                Arc::clone(&DESCRIPTOR)
            }
        
            pub fn new(world: Arc<World>) -> Arc<dyn Treatment> {
                let data_input = unbounded();
                let treatment = Arc::new(Self {
                    world,
                    neg_infinity: RwLock::new(<$output_rust_type>::MIN),
                    pos_infinity: RwLock::new(<$output_rust_type>::MAX),
                    nan: RwLock::new(<$output_rust_type>::default()),
                    data_output_transmitters: RwLock::new(Vec::new()),
                    data_input_sender: data_input.0,
                    data_input_receiver: data_input.1,
                    auto_reference: RwLock::new(Weak::new()),
                });
        
                *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);
        
                treatment
            }
        
            async fn add(&self) -> ResultStatus {
        
                let neg_infinity = *self.neg_infinity.read().unwrap();
                let pos_infinity = *self.pos_infinity.read().unwrap();
                let nan = *self.nan.read().unwrap();
                let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();
        
                while let Ok(value) = self.data_input_receiver.recv().await {
        
                    let output_value =
                        if value.is_finite() { value as $output_rust_type }
                        else if value.is_nan() { nan }
                        else if value.is_sign_positive() { pos_infinity }
                        else /*if value.is_sign_negative()*/ { neg_infinity }
                        ;
        
                    for transmitter in &inputs_to_fill {
                        match transmitter {
                            Transmitter::$output_mel_type(sender) => sender.send(output_value).await.unwrap(),
                            _ => panic!("{} sender expected!", std::any::type_name::<$output_rust_type>())
                        };
                    }
                }
        
                for transmitter in inputs_to_fill {
                    match transmitter {
                        Transmitter::$output_mel_type(sender) => sender.close(),
                        _ => panic!("{} sender expected!", std::any::type_name::<$output_rust_type>())
                    };
                }
        
                ResultStatus::default()
            }
        }

        impl Treatment for $name {

            fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
                Self::descriptor()
            }
        
            fn set_parameter(&self, param: &str, value: &Value) {
                
                match param {
                    "neg_infinity" => {
                        match value {
                            Value::$output_mel_type(value) => *self.neg_infinity.write().unwrap() = *value,
                            _ => panic!("Unexpected value type for 'neg_infinity'."),
                        }
                    },
                    "pos_infinity" => {
                        match value {
                            Value::$output_mel_type(value) => *self.pos_infinity.write().unwrap() = *value,
                            _ => panic!("Unexpected value type for 'pos_infinity'."),
                        }
                    },
                    "nan" => {
                        match value {
                            Value::$output_mel_type(value) => *self.nan.write().unwrap() = *value,
                            _ => panic!("Unexpected value type for 'nan'."),
                        }
                    },
                    _ => panic!("No parameter '{}' exists.", param)
                }
            }
        
            fn set_model(&self, name: &str, model: &Arc<dyn Model>) {
                panic!("No model expected.")
            }
        
            fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>) {
                
                match output_name {
                    "value" => self.data_output_transmitters.write().unwrap().extend(transmitter),
                    _ => panic!("No output '{}' exists.", output_name)
                }
            }
        
            fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {
        
                let mut hashmap = HashMap::new();
        
                hashmap.insert("value".to_string(), vec![Transmitter::$input_mel_type(self.data_input_sender.clone())]);
        
                hashmap
            }
        
            fn prepare(&self) -> Vec<TrackFuture> {
        
                let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
                let future = Box::new(Box::pin(async move { auto_self.add().await }));
        
                vec![future]
            }
            
        }
    };
}

// Conversions for f32
impl_ScalarFloatToInteger!(ScalarF32ToU8, "ScalarF32ToU8", f32, F32, u8, U8);
impl_ScalarFloatToInteger!(ScalarF32ToU16, "ScalarF32ToU16", f32, F32, u16, U16);
impl_ScalarFloatToInteger!(ScalarF32ToU32, "ScalarF32ToU32", f32, F32, u32, U32);
impl_ScalarFloatToInteger!(ScalarF32ToU64, "ScalarF32ToU64", f32, F32, u64, U64);
impl_ScalarFloatToInteger!(ScalarF32ToU128, "ScalarF32ToU128", f32, F32, u128, U128);
impl_ScalarFloatToInteger!(ScalarF32ToI8, "ScalarF32ToI8", f32, F32, i8, I8);
impl_ScalarFloatToInteger!(ScalarF32ToI16, "ScalarF32ToI16", f32, F32, i16, I16);
impl_ScalarFloatToInteger!(ScalarF32ToI32, "ScalarF32ToI32", f32, F32, i32, I32);
impl_ScalarFloatToInteger!(ScalarF32ToI64, "ScalarF32ToI64", f32, F32, i64, I64);
impl_ScalarFloatToInteger!(ScalarF32ToI128, "ScalarF32ToI128", f32, F32, i128, I128);

// Conversions for f64
impl_ScalarFloatToInteger!(ScalarF64ToU8, "ScalarF64ToU8", f64, F64, u8, U8);
impl_ScalarFloatToInteger!(ScalarF64ToU16, "ScalarF64ToU16", f64, F64, u16, U16);
impl_ScalarFloatToInteger!(ScalarF64ToU32, "ScalarF64ToU32", f64, F64, u32, U32);
impl_ScalarFloatToInteger!(ScalarF64ToU64, "ScalarF64ToU64", f64, F64, u64, U64);
impl_ScalarFloatToInteger!(ScalarF64ToU128, "ScalarF64ToU128", f64, F64, u128, U128);
impl_ScalarFloatToInteger!(ScalarF64ToI8, "ScalarF64ToI8", f64, F64, i8, I8);
impl_ScalarFloatToInteger!(ScalarF64ToI16, "ScalarF64ToI16", f64, F64, i16, I16);
impl_ScalarFloatToInteger!(ScalarF64ToI32, "ScalarF64ToI32", f64, F64, i32, I32);
impl_ScalarFloatToInteger!(ScalarF64ToI64, "ScalarF64ToI64", f64, F64, i64, I64);
impl_ScalarFloatToInteger!(ScalarF64ToI128, "ScalarF64ToI128", f64, F64, i128, I128);

/*
    FOR DEVELOPERS

The lines can be regenerated as will using the following script:

#!/bin/bash

FLOAT_TYPES="f32 f64"
INT_TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128"

for FLOAT in $FLOAT_TYPES
do
    UC_FLOAT=${FLOAT^}
    
    echo "// Conversions for $FLOAT"
    
    for INT in $INT_TYPES
    do
           UC_INT=${INT^}
           
           #echo "impl_ScalarFloatToInteger!(Scalar${UC_FLOAT}To${UC_INT}, \"Scalar${UC_FLOAT}To${UC_INT}\", $FLOAT, $UC_FLOAT, $INT, $UC_INT);"
           echo "c.treatments.insert(&(Scalar${UC_FLOAT}To${UC_INT}::descriptor() as Arc<dyn TreatmentDescriptor>));"
    done
    
    echo
done
    
```
    
*/
