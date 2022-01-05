
use super::super::prelude::*;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! impl_CastScalar {
    ($name:ident, $mel_name:expr, $input_rust_type:ty, $input_mel_type:ident, $output_rust_type:ty, $output_mel_type:ident) => {
        pub struct $name {

            world: Arc<World>,

            truncate: AtomicBool,
            or_default: RwLock<$output_rust_type>,
        
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
                            core_identifier!("cast";$mel_name),
                            models![],
                            treatment_sources![],
                            vec![
                                parameter!("truncate", Scalar, Bool, Some(Value::Bool(true))),
                                parameter!("or_default", Scalar, $output_mel_type, Some(Value::$output_mel_type(<$output_rust_type>::default()))),
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
                    truncate: AtomicBool::new(true),
                    or_default: RwLock::new(<$output_rust_type>::default()),
                    data_output_transmitters: RwLock::new(Vec::new()),
                    data_input_sender: data_input.0,
                    data_input_receiver: data_input.1,
                    auto_reference: RwLock::new(Weak::new()),
                });
        
                *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);
        
                treatment
            }
        
            async fn cast_truncate(&self) -> ResultStatus {
        
                let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();
        
                while let Ok(data) = self.data_input_receiver.recv().await {
        
                    let output_data = data as $output_rust_type;
        
                    for transmitter in &inputs_to_fill {
                        match transmitter {
                            Transmitter::$output_mel_type(sender) => sender.send(output_data).await.unwrap(),
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

            async fn cast_default(&self) -> ResultStatus {
        
                let default = *self.or_default.read().unwrap();
                let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();
        
                while let Ok(data) = self.data_input_receiver.recv().await {
        
                    //let output_data = data as $output_rust_type;
                    let output_data = if let Ok(casted_data) = <$output_rust_type>::try_from(data) {
                        casted_data
                    }
                    else {
                        default
                    };
        
                    for transmitter in &inputs_to_fill {
                        match transmitter {
                            Transmitter::$output_mel_type(sender) => sender.send(output_data).await.unwrap(),
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
                    "truncate" => {
                        match value {
                            Value::Bool(truncate) => self.truncate.store(*truncate, Ordering::Relaxed),
                            _ => panic!("Unexpected value type for 'truncate'."),
                        }
                    },
                    "or_default" => {
                        match value {
                            Value::$output_mel_type(value) => *self.or_default.write().unwrap() = *value,
                            _ => panic!("Unexpected value type for 'or_default'."),
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
                
                let future = match self.truncate.load(Ordering::Relaxed) {
                    true => Box::new(Box::pin(async move { auto_self.cast_truncate().await })) as TrackFuture,
                    false => Box::new(Box::pin(async move { auto_self.cast_default().await })) as TrackFuture,
                };
        
                vec![future]
            }
            
        }
    };
}

// Lossy casts for u8
impl_CastScalar!(CastScalarU8ToI8, "CastScalarU8ToI8", u8, U8, i8, I8);

// Lossy casts for u16
impl_CastScalar!(CastScalarU16ToU8, "CastScalarU16ToU8", u16, U16, u8, U8);
impl_CastScalar!(CastScalarU16ToI8, "CastScalarU16ToI8", u16, U16, i8, I8);
impl_CastScalar!(CastScalarU16ToI16, "CastScalarU16ToI16", u16, U16, i16, I16);

// Lossy casts for u32
impl_CastScalar!(CastScalarU32ToU8, "CastScalarU32ToU8", u32, U32, u8, U8);
impl_CastScalar!(CastScalarU32ToU16, "CastScalarU32ToU16", u32, U32, u16, U16);
impl_CastScalar!(CastScalarU32ToI8, "CastScalarU32ToI8", u32, U32, i8, I8);
impl_CastScalar!(CastScalarU32ToI16, "CastScalarU32ToI16", u32, U32, i16, I16);
impl_CastScalar!(CastScalarU32ToI32, "CastScalarU32ToI32", u32, U32, i32, I32);

// Lossy casts for u64
impl_CastScalar!(CastScalarU64ToU8, "CastScalarU64ToU8", u64, U64, u8, U8);
impl_CastScalar!(CastScalarU64ToU16, "CastScalarU64ToU16", u64, U64, u16, U16);
impl_CastScalar!(CastScalarU64ToU32, "CastScalarU64ToU32", u64, U64, u32, U32);
impl_CastScalar!(CastScalarU64ToI8, "CastScalarU64ToI8", u64, U64, i8, I8);
impl_CastScalar!(CastScalarU64ToI16, "CastScalarU64ToI16", u64, U64, i16, I16);
impl_CastScalar!(CastScalarU64ToI32, "CastScalarU64ToI32", u64, U64, i32, I32);
impl_CastScalar!(CastScalarU64ToI64, "CastScalarU64ToI64", u64, U64, i64, I64);

// Lossy casts for u128
impl_CastScalar!(CastScalarU128ToU8, "CastScalarU128ToU8", u128, U128, u8, U8);
impl_CastScalar!(CastScalarU128ToU16, "CastScalarU128ToU16", u128, U128, u16, U16);
impl_CastScalar!(CastScalarU128ToU32, "CastScalarU128ToU32", u128, U128, u32, U32);
impl_CastScalar!(CastScalarU128ToU64, "CastScalarU128ToU64", u128, U128, u64, U64);
impl_CastScalar!(CastScalarU128ToI8, "CastScalarU128ToI8", u128, U128, i8, I8);
impl_CastScalar!(CastScalarU128ToI16, "CastScalarU128ToI16", u128, U128, i16, I16);
impl_CastScalar!(CastScalarU128ToI32, "CastScalarU128ToI32", u128, U128, i32, I32);
impl_CastScalar!(CastScalarU128ToI64, "CastScalarU128ToI64", u128, U128, i64, I64);
impl_CastScalar!(CastScalarU128ToI128, "CastScalarU128ToI128", u128, U128, i128, I128);

// Lossy casts for i8
impl_CastScalar!(CastScalarI8ToU8, "CastScalarI8ToU8", i8, I8, u8, U8);
impl_CastScalar!(CastScalarI8ToU16, "CastScalarI8ToU16", i8, I8, u16, U16);
impl_CastScalar!(CastScalarI8ToU32, "CastScalarI8ToU32", i8, I8, u32, U32);
impl_CastScalar!(CastScalarI8ToU64, "CastScalarI8ToU64", i8, I8, u64, U64);
impl_CastScalar!(CastScalarI8ToU128, "CastScalarI8ToU128", i8, I8, u128, U128);

// Lossy casts for i16
impl_CastScalar!(CastScalarI16ToU8, "CastScalarI16ToU8", i16, I16, u8, U8);
impl_CastScalar!(CastScalarI16ToU16, "CastScalarI16ToU16", i16, I16, u16, U16);
impl_CastScalar!(CastScalarI16ToU32, "CastScalarI16ToU32", i16, I16, u32, U32);
impl_CastScalar!(CastScalarI16ToU64, "CastScalarI16ToU64", i16, I16, u64, U64);
impl_CastScalar!(CastScalarI16ToU128, "CastScalarI16ToU128", i16, I16, u128, U128);
impl_CastScalar!(CastScalarI16ToI8, "CastScalarI16ToI8", i16, I16, i8, I8);

// Lossy casts for i32
impl_CastScalar!(CastScalarI32ToU8, "CastScalarI32ToU8", i32, I32, u8, U8);
impl_CastScalar!(CastScalarI32ToU16, "CastScalarI32ToU16", i32, I32, u16, U16);
impl_CastScalar!(CastScalarI32ToU32, "CastScalarI32ToU32", i32, I32, u32, U32);
impl_CastScalar!(CastScalarI32ToU64, "CastScalarI32ToU64", i32, I32, u64, U64);
impl_CastScalar!(CastScalarI32ToU128, "CastScalarI32ToU128", i32, I32, u128, U128);
impl_CastScalar!(CastScalarI32ToI8, "CastScalarI32ToI8", i32, I32, i8, I8);
impl_CastScalar!(CastScalarI32ToI16, "CastScalarI32ToI16", i32, I32, i16, I16);

// Lossy casts for i64
impl_CastScalar!(CastScalarI64ToU8, "CastScalarI64ToU8", i64, I64, u8, U8);
impl_CastScalar!(CastScalarI64ToU16, "CastScalarI64ToU16", i64, I64, u16, U16);
impl_CastScalar!(CastScalarI64ToU32, "CastScalarI64ToU32", i64, I64, u32, U32);
impl_CastScalar!(CastScalarI64ToU64, "CastScalarI64ToU64", i64, I64, u64, U64);
impl_CastScalar!(CastScalarI64ToU128, "CastScalarI64ToU128", i64, I64, u128, U128);
impl_CastScalar!(CastScalarI64ToI8, "CastScalarI64ToI8", i64, I64, i8, I8);
impl_CastScalar!(CastScalarI64ToI16, "CastScalarI64ToI16", i64, I64, i16, I16);
impl_CastScalar!(CastScalarI64ToI32, "CastScalarI64ToI32", i64, I64, i32, I32);

// Lossy casts for i128
impl_CastScalar!(CastScalarI128ToU8, "CastScalarI128ToU8", i128, I128, u8, U8);
impl_CastScalar!(CastScalarI128ToU16, "CastScalarI128ToU16", i128, I128, u16, U16);
impl_CastScalar!(CastScalarI128ToU32, "CastScalarI128ToU32", i128, I128, u32, U32);
impl_CastScalar!(CastScalarI128ToU64, "CastScalarI128ToU64", i128, I128, u64, U64);
impl_CastScalar!(CastScalarI128ToU128, "CastScalarI128ToU128", i128, I128, u128, U128);
impl_CastScalar!(CastScalarI128ToI8, "CastScalarI128ToI8", i128, I128, i8, I8);
impl_CastScalar!(CastScalarI128ToI16, "CastScalarI128ToI16", i128, I128, i16, I16);
impl_CastScalar!(CastScalarI128ToI32, "CastScalarI128ToI32", i128, I128, i32, I32);
impl_CastScalar!(CastScalarI128ToI64, "CastScalarI128ToI64", i128, I128, i64, I64);

/*
    FOR DEVELOPERS

The lines about u/i* casts can be regenerated as will using the following script:

```
#!/bin/bash

TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128"

for TYPE in $TYPES
do
    TYPE_SIG=`echo $TYPE | grep -o [a-z]`
    TYPE_SIZE=`echo $TYPE | grep -oE [0-9]+`
    
    QUALIFIED_TYPES="$TYPES"
    if [ $TYPE_SIG == 'i' ]
    then
        QUALIFIED_TYPES=`echo $QUALIFIED_TYPES | sed -E s/u[0-9]+//g`
    fi
    
    while [ $TYPE_SIZE -ge 8 ]
    do
        QUALIFIED_TYPES=`echo $QUALIFIED_TYPES | sed s/[a-z]$TYPE_SIZE//g`
        TYPE_SIZE=`expr $TYPE_SIZE / 2`
    done
    
    DISQUALIFIED_TYPES=`echo $TYPES | sed s/$TYPE//g`
    for QUALIFIED_TYPE in $QUALIFIED_TYPES
    do
        DISQUALIFIED_TYPES=`echo $DISQUALIFIED_TYPES | sed s/$QUALIFIED_TYPE//g`
    done
    
    echo "// Lossy casts for $TYPE"
    
    UPPER_CASE_TYPE=`echo $TYPE | tr '[:lower:]' '[:upper:]'`
    for CAST_TYPE in $DISQUALIFIED_TYPES
    do
        UPPER_CASE_CAST_TYPE=`echo $CAST_TYPE | tr '[:lower:]' '[:upper:]'`
        
        echo "impl_CastScalar!(CastScalar${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}, \"CastScalar${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}\", $TYPE, $UPPER_CASE_TYPE, $CAST_TYPE, $UPPER_CASE_CAST_TYPE);"
        #echo "c.treatments.insert(&(CastScalar${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}::descriptor() as Arc<dyn TreatmentDescriptor>));"
    done
    
    echo 
done
```
    
*/