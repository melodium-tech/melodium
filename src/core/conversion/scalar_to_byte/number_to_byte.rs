
use super::super::super::prelude::*;

macro_rules! impl_ScalarToByte {
    ($name:ident, $mel_name:expr, $input_rust_type:ty, $input_mel_type:ident) => {
        struct $name {

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
                            core_identifier!("conversion";$mel_name),
                            models![],
                            treatment_sources![],
                            vec![],
                            vec![
                                input!("value",Scalar,$input_mel_type,Stream)
                            ],
                            vec![
                                output!("data",Scalar,Byte,Stream)
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
        
            async fn to_bytes(&self) -> ResultStatus {
        
                let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();
        
                while let Ok(value) = self.data_input_receiver.recv().await {
        
                    let output_data = value.to_be_bytes();
        
                    for transmitter in &inputs_to_fill {
                        match transmitter {
                            Transmitter::Byte(sender) => for byte in output_data { sender.send(byte).await.unwrap() },
                            _ => panic!("{} sender expected!", std::any::type_name::<u8>())
                        };
                    }
                }
        
                for transmitter in inputs_to_fill {
                    match transmitter {
                        Transmitter::Byte(sender) => sender.close(),
                        _ => panic!("{} sender expected!", std::any::type_name::<u8>())
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
                    "data" => self.data_output_transmitters.write().unwrap().extend(transmitter),
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
                let future = Box::new(Box::pin(async move { auto_self.to_bytes().await }));
        
                vec![future]
            }
            
        }
    };
}

impl_ScalarToByte!(U8ToByte, "U8ToByte", u8, U8);
impl_ScalarToByte!(U16ToByte, "U16ToByte", u16, U16);
impl_ScalarToByte!(U32ToByte, "U32ToByte", u32, U32);
impl_ScalarToByte!(U64ToByte, "U64ToByte", u64, U64);
impl_ScalarToByte!(U128ToByte, "U128ToByte", u128, U128);
impl_ScalarToByte!(I8ToByte, "I8ToByte", i8, I8);
impl_ScalarToByte!(I16ToByte, "I16ToByte", i16, I16);
impl_ScalarToByte!(I32ToByte, "I32ToByte", i32, I32);
impl_ScalarToByte!(I64ToByte, "I64ToByte", i64, I64);
impl_ScalarToByte!(I128ToByte, "I128ToByte", i128, I128);
impl_ScalarToByte!(F32ToByte, "F32ToByte", f32, F32);
impl_ScalarToByte!(F64ToByte, "F64ToByte", f64, F64);

/*
    FOR DEVELOPERS

The lines above can be regenerated as will using the following script:

```
#!/bin/bash

TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64"

for TYPE in $TYPES
do
    UPPER_CASE_TYPE=${TYPE^}
    echo "impl_ScalarToByte!(${UPPER_CASE_TYPE}ToByte, \"${UPPER_CASE_TYPE}ToByte\", $TYPE, $UPPER_CASE_TYPE);"
    #echo "c.treatments.insert(&(Vector${UPPER_CASE_TYPE}ToVectorString::descriptor() as Arc<dyn TreatmentDescriptor>));"

done
```
    
*/

