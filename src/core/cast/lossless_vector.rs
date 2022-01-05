
use super::super::prelude::*;
use std::iter::Iterator;

macro_rules! impl_CastVector {
    ($name:ident, $mel_name:expr, $input_rust_type:ty, $input_mel_type:ident, $input_trans_type:ident, $output_rust_type:ty, $output_mel_type:ident, $output_trans_type:ident) => {
        pub struct $name {

            world: Arc<World>,
        
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
                            vec![],
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

// Casts for f32 and f64
impl_CastVector!(CastVectorF32ToF64, "CastVectorF32ToF64", f32, F32, VecF32, f64, F64, VecF64);
impl_CastVector!(CastVectorF64ToF32, "CastVectorF64ToF32", f64, F64, VecF64, f32, F32, VecF32);

// Casts for u8
impl_CastVector!(CastVectorU8ToU16, "CastVectorU8ToU16", u8, U8, VecU8, u16, U16, VecU16);
impl_CastVector!(CastVectorU8ToU32, "CastVectorU8ToU32", u8, U8, VecU8, u32, U32, VecU32);
impl_CastVector!(CastVectorU8ToU64, "CastVectorU8ToU64", u8, U8, VecU8, u64, U64, VecU64);
impl_CastVector!(CastVectorU8ToU128, "CastVectorU8ToU128", u8, U8, VecU8, u128, U128, VecU128);
impl_CastVector!(CastVectorU8ToI16, "CastVectorU8ToI16", u8, U8, VecU8, i16, I16, VecI16);
impl_CastVector!(CastVectorU8ToI32, "CastVectorU8ToI32", u8, U8, VecU8, i32, I32, VecI32);
impl_CastVector!(CastVectorU8ToI64, "CastVectorU8ToI64", u8, U8, VecU8, i64, I64, VecI64);
impl_CastVector!(CastVectorU8ToI128, "CastVectorU8ToI128", u8, U8, VecU8, i128, I128, VecI128);

// Casts for u16
impl_CastVector!(CastVectorU16ToU32, "CastVectorU16ToU32", u16, U16, VecU16, u32, U32, VecU32);
impl_CastVector!(CastVectorU16ToU64, "CastVectorU16ToU64", u16, U16, VecU16, u64, U64, VecU64);
impl_CastVector!(CastVectorU16ToU128, "CastVectorU16ToU128", u16, U16, VecU16, u128, U128, VecU128);
impl_CastVector!(CastVectorU16ToI32, "CastVectorU16ToI32", u16, U16, VecU16, i32, I32, VecI32);
impl_CastVector!(CastVectorU16ToI64, "CastVectorU16ToI64", u16, U16, VecU16, i64, I64, VecI64);
impl_CastVector!(CastVectorU16ToI128, "CastVectorU16ToI128", u16, U16, VecU16, i128, I128, VecI128);

// Casts for u32
impl_CastVector!(CastVectorU32ToU64, "CastVectorU32ToU64", u32, U32, VecU32, u64, U64, VecU64);
impl_CastVector!(CastVectorU32ToU128, "CastVectorU32ToU128", u32, U32, VecU32, u128, U128, VecU128);
impl_CastVector!(CastVectorU32ToI64, "CastVectorU32ToI64", u32, U32, VecU32, i64, I64, VecI64);
impl_CastVector!(CastVectorU32ToI128, "CastVectorU32ToI128", u32, U32, VecU32, i128, I128, VecI128);

// Casts for u64
impl_CastVector!(CastVectorU64ToU128, "CastVectorU64ToU128", u64, U64, VecU64, u128, U128, VecU128);
impl_CastVector!(CastVectorU64ToI128, "CastVectorU64ToI128", u64, U64, VecU64, i128, I128, VecI128);

// Casts for u128

// Casts for i8
impl_CastVector!(CastVectorI8ToI16, "CastVectorI8ToI16", i8, I8, VecI8, i16, I16, VecI16);
impl_CastVector!(CastVectorI8ToI32, "CastVectorI8ToI32", i8, I8, VecI8, i32, I32, VecI32);
impl_CastVector!(CastVectorI8ToI64, "CastVectorI8ToI64", i8, I8, VecI8, i64, I64, VecI64);
impl_CastVector!(CastVectorI8ToI128, "CastVectorI8ToI128", i8, I8, VecI8, i128, I128, VecI128);

// Casts for i16
impl_CastVector!(CastVectorI16ToI32, "CastVectorI16ToI32", i16, I16, VecI16, i32, I32, VecI32);
impl_CastVector!(CastVectorI16ToI64, "CastVectorI16ToI64", i16, I16, VecI16, i64, I64, VecI64);
impl_CastVector!(CastVectorI16ToI128, "CastVectorI16ToI128", i16, I16, VecI16, i128, I128, VecI128);

// Casts for i32
impl_CastVector!(CastVectorI32ToI64, "CastVectorI32ToI64", i32, I32, VecI32, i64, I64, VecI64);
impl_CastVector!(CastVectorI32ToI128, "CastVectorI32ToI128", i32, I32, VecI32, i128, I128, VecI128);

// Casts for i64
impl_CastVector!(CastVectorI64ToI128, "CastVectorI64ToI128", i64, I64, VecI64, i128, I128, VecI128);

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
    
    echo "// Casts for $TYPE"
    
    UPPER_CASE_TYPE=`echo $TYPE | tr '[:lower:]' '[:upper:]'`
    for CAST_TYPE in $QUALIFIED_TYPES
    do
        UPPER_CASE_CAST_TYPE=`echo $CAST_TYPE | tr '[:lower:]' '[:upper:]'`
        
        echo "impl_CastVector!(CastVector${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}, \"CastVector${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}\", $TYPE, $UPPER_CASE_TYPE, Vec$UPPER_CASE_TYPE, $CAST_TYPE, $UPPER_CASE_CAST_TYPE, Vec$UPPER_CASE_CAST_TYPE);"
        #echo "c.treatments.insert(&(CastVector${UPPER_CASE_TYPE}To${UPPER_CASE_CAST_TYPE}::descriptor() as Arc<dyn TreatmentDescriptor>));"
    done
    
    echo 
done
```
    
*/