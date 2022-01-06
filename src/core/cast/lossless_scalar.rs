
use super::super::prelude::*;

macro_rules! impl_CastScalar {
    ($name:ident, $mel_name:expr, $input_rust_type:ty, $input_mel_type:ident, $output_rust_type:ty, $output_mel_type:ident) => {
        pub struct $name {

            world: Arc<World>,
        
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
                            vec![],
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
                    data_output_transmitters: RwLock::new(Vec::new()),
                    data_input_sender: data_input.0,
                    data_input_receiver: data_input.1,
                    auto_reference: RwLock::new(Weak::new()),
                });
        
                *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);
        
                treatment
            }
        
            async fn cast(&self) -> ResultStatus {
        
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
        }

        impl Treatment for $name {

            fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
                Self::descriptor()
            }
        
            fn set_parameter(&self, param: &str, value: &Value) {
                
                panic!("No parameter expected.")
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
                let future = Box::new(Box::pin(async move { auto_self.cast().await }));
        
                vec![future]
            }
            
        }
    };
}

// Casts for f32 and f64
impl_CastScalar!(CastScalarF32ToF64, "CastScalarF32ToF64", f32, F32, f64, F64);
impl_CastScalar!(CastScalarF64ToF32, "CastScalarF64ToF32", f64, F64, f32, F32);

// Casts for u8
impl_CastScalar!(CastScalarU8ToU16, "CastScalarU8ToU16", u8, U8, u16, U16);
impl_CastScalar!(CastScalarU8ToU32, "CastScalarU8ToU32", u8, U8, u32, U32);
impl_CastScalar!(CastScalarU8ToU64, "CastScalarU8ToU64", u8, U8, u64, U64);
impl_CastScalar!(CastScalarU8ToU128, "CastScalarU8ToU128", u8, U8, u128, U128);
impl_CastScalar!(CastScalarU8ToI16, "CastScalarU8ToI16", u8, U8, i16, I16);
impl_CastScalar!(CastScalarU8ToI32, "CastScalarU8ToI32", u8, U8, i32, I32);
impl_CastScalar!(CastScalarU8ToI64, "CastScalarU8ToI64", u8, U8, i64, I64);
impl_CastScalar!(CastScalarU8ToI128, "CastScalarU8ToI128", u8, U8, i128, I128);
impl_CastScalar!(CastScalarU8ToF32, "CastScalarU8ToF32", u8, U8, f32, F32);
impl_CastScalar!(CastScalarU8ToF64, "CastScalarU8ToF64", u8, U8, f64, F64);

// Casts for u16
impl_CastScalar!(CastScalarU16ToU32, "CastScalarU16ToU32", u16, U16, u32, U32);
impl_CastScalar!(CastScalarU16ToU64, "CastScalarU16ToU64", u16, U16, u64, U64);
impl_CastScalar!(CastScalarU16ToU128, "CastScalarU16ToU128", u16, U16, u128, U128);
impl_CastScalar!(CastScalarU16ToI32, "CastScalarU16ToI32", u16, U16, i32, I32);
impl_CastScalar!(CastScalarU16ToI64, "CastScalarU16ToI64", u16, U16, i64, I64);
impl_CastScalar!(CastScalarU16ToI128, "CastScalarU16ToI128", u16, U16, i128, I128);
impl_CastScalar!(CastScalarU16ToF32, "CastScalarU16ToF32", u16, U16, f32, F32);
impl_CastScalar!(CastScalarU16ToF64, "CastScalarU16ToF64", u16, U16, f64, F64);

// Casts for u32
impl_CastScalar!(CastScalarU32ToU64, "CastScalarU32ToU64", u32, U32, u64, U64);
impl_CastScalar!(CastScalarU32ToU128, "CastScalarU32ToU128", u32, U32, u128, U128);
impl_CastScalar!(CastScalarU32ToI64, "CastScalarU32ToI64", u32, U32, i64, I64);
impl_CastScalar!(CastScalarU32ToI128, "CastScalarU32ToI128", u32, U32, i128, I128);
impl_CastScalar!(CastScalarU32ToF32, "CastScalarU32ToF32", u32, U32, f32, F32);
impl_CastScalar!(CastScalarU32ToF64, "CastScalarU32ToF64", u32, U32, f64, F64);

// Casts for u64
impl_CastScalar!(CastScalarU64ToU128, "CastScalarU64ToU128", u64, U64, u128, U128);
impl_CastScalar!(CastScalarU64ToI128, "CastScalarU64ToI128", u64, U64, i128, I128);
impl_CastScalar!(CastScalarU64ToF32, "CastScalarU64ToF32", u64, U64, f32, F32);
impl_CastScalar!(CastScalarU64ToF64, "CastScalarU64ToF64", u64, U64, f64, F64);

// Casts for u128
impl_CastScalar!(CastScalarU128ToF32, "CastScalarU128ToF32", u128, U128, f32, F32);
impl_CastScalar!(CastScalarU128ToF64, "CastScalarU128ToF64", u128, U128, f64, F64);

// Casts for i8
impl_CastScalar!(CastScalarI8ToI16, "CastScalarI8ToI16", i8, I8, i16, I16);
impl_CastScalar!(CastScalarI8ToI32, "CastScalarI8ToI32", i8, I8, i32, I32);
impl_CastScalar!(CastScalarI8ToI64, "CastScalarI8ToI64", i8, I8, i64, I64);
impl_CastScalar!(CastScalarI8ToI128, "CastScalarI8ToI128", i8, I8, i128, I128);
impl_CastScalar!(CastScalarI8ToF32, "CastScalarI8ToF32", i8, I8, f32, F32);
impl_CastScalar!(CastScalarI8ToF64, "CastScalarI8ToF64", i8, I8, f64, F64);

// Casts for i16
impl_CastScalar!(CastScalarI16ToI32, "CastScalarI16ToI32", i16, I16, i32, I32);
impl_CastScalar!(CastScalarI16ToI64, "CastScalarI16ToI64", i16, I16, i64, I64);
impl_CastScalar!(CastScalarI16ToI128, "CastScalarI16ToI128", i16, I16, i128, I128);
impl_CastScalar!(CastScalarI16ToF32, "CastScalarI16ToF32", i16, I16, f32, F32);
impl_CastScalar!(CastScalarI16ToF64, "CastScalarI16ToF64", i16, I16, f64, F64);

// Casts for i32
impl_CastScalar!(CastScalarI32ToI64, "CastScalarI32ToI64", i32, I32, i64, I64);
impl_CastScalar!(CastScalarI32ToI128, "CastScalarI32ToI128", i32, I32, i128, I128);
impl_CastScalar!(CastScalarI32ToF32, "CastScalarI32ToF32", i32, I32, f32, F32);
impl_CastScalar!(CastScalarI32ToF64, "CastScalarI32ToF64", i32, I32, f64, F64);

// Casts for i64
impl_CastScalar!(CastScalarI64ToI128, "CastScalarI64ToI128", i64, I64, i128, I128);
impl_CastScalar!(CastScalarI64ToF32, "CastScalarI64ToF32", i64, I64, f32, F32);
impl_CastScalar!(CastScalarI64ToF64, "CastScalarI64ToF64", i64, I64, f64, F64);

// Casts for i128
impl_CastScalar!(CastScalarI128ToF32, "CastScalarI128ToF32", i128, I128, f32, F32);
impl_CastScalar!(CastScalarI128ToF64, "CastScalarI128ToF64", i128, I128, f64, F64);


// Casts for i128

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
    
    QUALIFIED_TYPES="$QUALIFIED_TYPES f32 f64"
    
    echo "// Casts for $TYPE"
    
    UPPER_CASE_TYPE=`echo $TYPE | tr '[:lower:]' '[:upper:]'`
    for CAST_TYPE in $QUALIFIED_TYPES
    do
        UPPER_CASE_CAST_TYPE=`echo $CAST_TYPE | tr '[:lower:]' '[:upper:]'`
        
        echo "impl_CastScalar!(CastScalar${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}, \"CastScalar${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}\", $TYPE, $UPPER_CASE_TYPE, $CAST_TYPE, $UPPER_CASE_CAST_TYPE);"
        #echo "c.treatments.insert(&(CastScalar${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}::descriptor() as Arc<dyn TreatmentDescriptor>));"
    done
    
    echo 
done
```
    
*/