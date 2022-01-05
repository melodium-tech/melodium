
use super::super::prelude::*;
use std::iter::Iterator;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! impl_CastVector {
    ($name:ident, $mel_name:expr, $input_rust_type:ty, $input_mel_type:ident, $input_trans_type:ident, $output_rust_type:ty, $output_mel_type:ident, $output_trans_type:ident) => {
        pub struct $name {

            world: Arc<World>,

            truncate: AtomicBool,
            or_default: RwLock<$output_rust_type>,
        
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
                            core_identifier!("cast";$mel_name),
                            models![],
                            treatment_sources![],
                            vec![
                                parameter!("truncate", Scalar, Bool, Some(Value::Bool(true))),
                                parameter!("or_default", Scalar, $output_mel_type, Some(Value::$output_mel_type(<$output_rust_type>::default()))),
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
        
                    let output_data : Vec<$output_rust_type> = data.iter().map(|v| *v as $output_rust_type).collect();
        
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

            async fn cast_default(&self) -> ResultStatus {
        
                let default = *self.or_default.read().unwrap();
                let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();
        
                while let Ok(data) = self.data_input_receiver.recv().await {
        
                    //let output_data : Vec<$output_rust_type> = data.iter().map(|v| *v as $output_rust_type).collect();

                    let output_data : Vec<$output_rust_type> = data.iter().map(|v| {
                        if let Ok(casted_data) = <$output_rust_type>::try_from(*v) {
                            casted_data
                        }
                        else {
                            default
                        }
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
        
                hashmap.insert("value".to_string(), vec![Transmitter::$input_trans_type(self.data_input_sender.clone())]);
        
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
impl_CastVector!(CastVectorU8ToI8, "CastVectorU8ToI8", u8, U8, VecU8, i8, I8, VecI8);

// Lossy casts for u16
impl_CastVector!(CastVectorU16ToU8, "CastVectorU16ToU8", u16, U16, VecU16, u8, U8, VecU8);
impl_CastVector!(CastVectorU16ToI8, "CastVectorU16ToI8", u16, U16, VecU16, i8, I8, VecI8);
impl_CastVector!(CastVectorU16ToI16, "CastVectorU16ToI16", u16, U16, VecU16, i16, I16, VecI16);

// Lossy casts for u32
impl_CastVector!(CastVectorU32ToU8, "CastVectorU32ToU8", u32, U32, VecU32, u8, U8, VecU8);
impl_CastVector!(CastVectorU32ToU16, "CastVectorU32ToU16", u32, U32, VecU32, u16, U16, VecU16);
impl_CastVector!(CastVectorU32ToI8, "CastVectorU32ToI8", u32, U32, VecU32, i8, I8, VecI8);
impl_CastVector!(CastVectorU32ToI16, "CastVectorU32ToI16", u32, U32, VecU32, i16, I16, VecI16);
impl_CastVector!(CastVectorU32ToI32, "CastVectorU32ToI32", u32, U32, VecU32, i32, I32, VecI32);

// Lossy casts for u64
impl_CastVector!(CastVectorU64ToU8, "CastVectorU64ToU8", u64, U64, VecU64, u8, U8, VecU8);
impl_CastVector!(CastVectorU64ToU16, "CastVectorU64ToU16", u64, U64, VecU64, u16, U16, VecU16);
impl_CastVector!(CastVectorU64ToU32, "CastVectorU64ToU32", u64, U64, VecU64, u32, U32, VecU32);
impl_CastVector!(CastVectorU64ToI8, "CastVectorU64ToI8", u64, U64, VecU64, i8, I8, VecI8);
impl_CastVector!(CastVectorU64ToI16, "CastVectorU64ToI16", u64, U64, VecU64, i16, I16, VecI16);
impl_CastVector!(CastVectorU64ToI32, "CastVectorU64ToI32", u64, U64, VecU64, i32, I32, VecI32);
impl_CastVector!(CastVectorU64ToI64, "CastVectorU64ToI64", u64, U64, VecU64, i64, I64, VecI64);

// Lossy casts for u128
impl_CastVector!(CastVectorU128ToU8, "CastVectorU128ToU8", u128, U128, VecU128, u8, U8, VecU8);
impl_CastVector!(CastVectorU128ToU16, "CastVectorU128ToU16", u128, U128, VecU128, u16, U16, VecU16);
impl_CastVector!(CastVectorU128ToU32, "CastVectorU128ToU32", u128, U128, VecU128, u32, U32, VecU32);
impl_CastVector!(CastVectorU128ToU64, "CastVectorU128ToU64", u128, U128, VecU128, u64, U64, VecU64);
impl_CastVector!(CastVectorU128ToI8, "CastVectorU128ToI8", u128, U128, VecU128, i8, I8, VecI8);
impl_CastVector!(CastVectorU128ToI16, "CastVectorU128ToI16", u128, U128, VecU128, i16, I16, VecI16);
impl_CastVector!(CastVectorU128ToI32, "CastVectorU128ToI32", u128, U128, VecU128, i32, I32, VecI32);
impl_CastVector!(CastVectorU128ToI64, "CastVectorU128ToI64", u128, U128, VecU128, i64, I64, VecI64);
impl_CastVector!(CastVectorU128ToI128, "CastVectorU128ToI128", u128, U128, VecU128, i128, I128, VecI128);

// Lossy casts for i8
impl_CastVector!(CastVectorI8ToU8, "CastVectorI8ToU8", i8, I8, VecI8, u8, U8, VecU8);
impl_CastVector!(CastVectorI8ToU16, "CastVectorI8ToU16", i8, I8, VecI8, u16, U16, VecU16);
impl_CastVector!(CastVectorI8ToU32, "CastVectorI8ToU32", i8, I8, VecI8, u32, U32, VecU32);
impl_CastVector!(CastVectorI8ToU64, "CastVectorI8ToU64", i8, I8, VecI8, u64, U64, VecU64);
impl_CastVector!(CastVectorI8ToU128, "CastVectorI8ToU128", i8, I8, VecI8, u128, U128, VecU128);

// Lossy casts for i16
impl_CastVector!(CastVectorI16ToU8, "CastVectorI16ToU8", i16, I16, VecI16, u8, U8, VecU8);
impl_CastVector!(CastVectorI16ToU16, "CastVectorI16ToU16", i16, I16, VecI16, u16, U16, VecU16);
impl_CastVector!(CastVectorI16ToU32, "CastVectorI16ToU32", i16, I16, VecI16, u32, U32, VecU32);
impl_CastVector!(CastVectorI16ToU64, "CastVectorI16ToU64", i16, I16, VecI16, u64, U64, VecU64);
impl_CastVector!(CastVectorI16ToU128, "CastVectorI16ToU128", i16, I16, VecI16, u128, U128, VecU128);
impl_CastVector!(CastVectorI16ToI8, "CastVectorI16ToI8", i16, I16, VecI16, i8, I8, VecI8);

// Lossy casts for i32
impl_CastVector!(CastVectorI32ToU8, "CastVectorI32ToU8", i32, I32, VecI32, u8, U8, VecU8);
impl_CastVector!(CastVectorI32ToU16, "CastVectorI32ToU16", i32, I32, VecI32, u16, U16, VecU16);
impl_CastVector!(CastVectorI32ToU32, "CastVectorI32ToU32", i32, I32, VecI32, u32, U32, VecU32);
impl_CastVector!(CastVectorI32ToU64, "CastVectorI32ToU64", i32, I32, VecI32, u64, U64, VecU64);
impl_CastVector!(CastVectorI32ToU128, "CastVectorI32ToU128", i32, I32, VecI32, u128, U128, VecU128);
impl_CastVector!(CastVectorI32ToI8, "CastVectorI32ToI8", i32, I32, VecI32, i8, I8, VecI8);
impl_CastVector!(CastVectorI32ToI16, "CastVectorI32ToI16", i32, I32, VecI32, i16, I16, VecI16);

// Lossy casts for i64
impl_CastVector!(CastVectorI64ToU8, "CastVectorI64ToU8", i64, I64, VecI64, u8, U8, VecU8);
impl_CastVector!(CastVectorI64ToU16, "CastVectorI64ToU16", i64, I64, VecI64, u16, U16, VecU16);
impl_CastVector!(CastVectorI64ToU32, "CastVectorI64ToU32", i64, I64, VecI64, u32, U32, VecU32);
impl_CastVector!(CastVectorI64ToU64, "CastVectorI64ToU64", i64, I64, VecI64, u64, U64, VecU64);
impl_CastVector!(CastVectorI64ToU128, "CastVectorI64ToU128", i64, I64, VecI64, u128, U128, VecU128);
impl_CastVector!(CastVectorI64ToI8, "CastVectorI64ToI8", i64, I64, VecI64, i8, I8, VecI8);
impl_CastVector!(CastVectorI64ToI16, "CastVectorI64ToI16", i64, I64, VecI64, i16, I16, VecI16);
impl_CastVector!(CastVectorI64ToI32, "CastVectorI64ToI32", i64, I64, VecI64, i32, I32, VecI32);

// Lossy casts for i128
impl_CastVector!(CastVectorI128ToU8, "CastVectorI128ToU8", i128, I128, VecI128, u8, U8, VecU8);
impl_CastVector!(CastVectorI128ToU16, "CastVectorI128ToU16", i128, I128, VecI128, u16, U16, VecU16);
impl_CastVector!(CastVectorI128ToU32, "CastVectorI128ToU32", i128, I128, VecI128, u32, U32, VecU32);
impl_CastVector!(CastVectorI128ToU64, "CastVectorI128ToU64", i128, I128, VecI128, u64, U64, VecU64);
impl_CastVector!(CastVectorI128ToU128, "CastVectorI128ToU128", i128, I128, VecI128, u128, U128, VecU128);
impl_CastVector!(CastVectorI128ToI8, "CastVectorI128ToI8", i128, I128, VecI128, i8, I8, VecI8);
impl_CastVector!(CastVectorI128ToI16, "CastVectorI128ToI16", i128, I128, VecI128, i16, I16, VecI16);
impl_CastVector!(CastVectorI128ToI32, "CastVectorI128ToI32", i128, I128, VecI128, i32, I32, VecI32);
impl_CastVector!(CastVectorI128ToI64, "CastVectorI128ToI64", i128, I128, VecI128, i64, I64, VecI64);


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
        
        echo "impl_CastVector!(CastVector${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}, \"CastVector${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}\", $TYPE, $UPPER_CASE_TYPE, Vec$UPPER_CASE_TYPE, $CAST_TYPE, $UPPER_CASE_CAST_TYPE, Vec$UPPER_CASE_CAST_TYPE);"
        #echo "c.treatments.insert(&(CastVector${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}::descriptor() as Arc<dyn TreatmentDescriptor>));"
    done
    
    echo 
done
```
    
*/