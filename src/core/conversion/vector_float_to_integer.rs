
use super::super::prelude::*;
use std::iter::Iterator;

macro_rules! impl_VectorFloatToInteger {
    ($name:ident, $mel_name:expr, $input_rust_type:ty, $input_mel_type:ident, $input_trans_type:ident, $output_rust_type:ty, $output_mel_type:ident, $output_trans_type:ident) => {
        struct $name {

            world: Arc<World>,
        
            neg_infinity: RwLock<$output_rust_type>,
            pos_infinity: RwLock<$output_rust_type>,
            nan: RwLock<$output_rust_type>,
        
            data_output_transmitters: RwLock<Vec<Transmitter>>,
            data_input_sender: Sender<Vec<$input_rust_type>>,
            data_input_receiver: Receiver<Vec<$input_rust_type>>,
        
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
                                input!("value",Vector,$input_mel_type,Stream)
                            ],
                            vec![
                                output!("value",Vector,$output_mel_type,Stream)
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
        
            async fn cast(&self) -> ResultStatus {
        
                let neg_infinity = *self.neg_infinity.read().unwrap();
                let pos_infinity = *self.pos_infinity.read().unwrap();
                let nan = *self.nan.read().unwrap();
                let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();
        
                while let Ok(data) = self.data_input_receiver.recv().await {
        
                    let output_data : Vec<$output_rust_type> = data.iter().map(|value| {
                        if value.is_finite() { *value as $output_rust_type }
                        else if value.is_nan() { nan }
                        else if value.is_sign_positive() { pos_infinity }
                        else /*if value.is_sign_negative()*/ { neg_infinity }
                    }).collect();
        
                    for transmitter in &inputs_to_fill {
                        match transmitter {
                            Transmitter::$output_trans_type(sender) => sender.send(output_data.clone()).await.unwrap(),
                            _ => panic!("{} sender expected!", std::any::type_name::<Vec<$output_rust_type>>())
                        };
                    }
                }
        
                for transmitter in inputs_to_fill {
                    match transmitter {
                        Transmitter::$output_trans_type(sender) => sender.close(),
                        _ => panic!("{} sender expected!", std::any::type_name::<Vec<$output_rust_type>>())
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
        
                hashmap.insert("value".to_string(), vec![Transmitter::$input_trans_type(self.data_input_sender.clone())]);
        
                hashmap
            }
        
            fn prepare(&self) -> Vec<TrackFuture> {
        
                let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
                let future = Box::new(Box::pin(async move { auto_self.cast().await }));
        
                vec![future]
            }
            
        }
    };
}

// Conversions for f32
impl_VectorFloatToInteger!(VectorF32ToU8, "VectorF32ToU8", f32, F32, VecF32, u8, U8, VecU8);
impl_VectorFloatToInteger!(VectorF32ToU16, "VectorF32ToU16", f32, F32, VecF32, u16, U16, VecU16);
impl_VectorFloatToInteger!(VectorF32ToU32, "VectorF32ToU32", f32, F32, VecF32, u32, U32, VecU32);
impl_VectorFloatToInteger!(VectorF32ToU64, "VectorF32ToU64", f32, F32, VecF32, u64, U64, VecU64);
impl_VectorFloatToInteger!(VectorF32ToU128, "VectorF32ToU128", f32, F32, VecF32, u128, U128, VecU128);
impl_VectorFloatToInteger!(VectorF32ToI8, "VectorF32ToI8", f32, F32, VecF32, i8, I8, VecI8);
impl_VectorFloatToInteger!(VectorF32ToI16, "VectorF32ToI16", f32, F32, VecF32, i16, I16, VecI16);
impl_VectorFloatToInteger!(VectorF32ToI32, "VectorF32ToI32", f32, F32, VecF32, i32, I32, VecI32);
impl_VectorFloatToInteger!(VectorF32ToI64, "VectorF32ToI64", f32, F32, VecF32, i64, I64, VecI64);
impl_VectorFloatToInteger!(VectorF32ToI128, "VectorF32ToI128", f32, F32, VecF32, i128, I128, VecI128);

// Conversions for f64
impl_VectorFloatToInteger!(VectorF64ToU8, "VectorF64ToU8", f64, F64, VecF64, u8, U8, VecU8);
impl_VectorFloatToInteger!(VectorF64ToU16, "VectorF64ToU16", f64, F64, VecF64, u16, U16, VecU16);
impl_VectorFloatToInteger!(VectorF64ToU32, "VectorF64ToU32", f64, F64, VecF64, u32, U32, VecU32);
impl_VectorFloatToInteger!(VectorF64ToU64, "VectorF64ToU64", f64, F64, VecF64, u64, U64, VecU64);
impl_VectorFloatToInteger!(VectorF64ToU128, "VectorF64ToU128", f64, F64, VecF64, u128, U128, VecU128);
impl_VectorFloatToInteger!(VectorF64ToI8, "VectorF64ToI8", f64, F64, VecF64, i8, I8, VecI8);
impl_VectorFloatToInteger!(VectorF64ToI16, "VectorF64ToI16", f64, F64, VecF64, i16, I16, VecI16);
impl_VectorFloatToInteger!(VectorF64ToI32, "VectorF64ToI32", f64, F64, VecF64, i32, I32, VecI32);
impl_VectorFloatToInteger!(VectorF64ToI64, "VectorF64ToI64", f64, F64, VecF64, i64, I64, VecI64);
impl_VectorFloatToInteger!(VectorF64ToI128, "VectorF64ToI128", f64, F64, VecF64, i128, I128, VecI128);

pub fn register(c: &mut CollectionPool) {

    // Conversions for f32
    c.treatments.insert(&(VectorF32ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF32ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF32ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF32ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF32ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF32ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF32ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF32ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF32ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF32ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));

    // Conversions for f64
    c.treatments.insert(&(VectorF64ToU8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF64ToU16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF64ToU32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF64ToU64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF64ToU128::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF64ToI8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF64ToI16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF64ToI32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF64ToI64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(VectorF64ToI128::descriptor() as Arc<dyn TreatmentDescriptor>));
}

/*
    FOR DEVELOPERS

The lines can be regenerated as will using the following script:

```
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
           
           echo "impl_VectorFloatToInteger!(Vector${UC_FLOAT}To${UC_INT}, \"Vector${UC_FLOAT}To${UC_INT}\", $FLOAT, $UC_FLOAT, Vec$UC_FLOAT, $INT, $UC_INT, Vec$UC_INT);"
           #echo "c.treatments.insert(&(Vector${UC_FLOAT}To${UC_INT}::descriptor() as Arc<dyn TreatmentDescriptor>));"
    done
    
    echo
done
```
    
*/

