
use crate::core::prelude::*;

macro_rules! impl_StreamToBlock {
    ($name:ident, $mel_name:expr, $rust_type:ty, $mel_input_type:ident, $mel_output_type:ident) => {
        struct $name {

            world: Arc<World>,
        
            data_output_transmitters: RwLock<Vec<Transmitter>>,
            data_input_sender: Sender<$rust_type>,
            data_input_receiver: Receiver<$rust_type>,
        
            auto_reference: RwLock<Weak<Self>>,
        
        }

        impl $name {

            pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {
        
                lazy_static! {
                    static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {
        
                        let rc_descriptor = CoreTreatmentDescriptor::new(
                            core_identifier!("flow";$mel_name),
                            models![],
                            treatment_sources![],
                            vec![],
                            vec![
                                input!("data",Scalar,$mel_input_type,Stream)
                            ],
                            vec![
                                output!("data",Vector,$mel_input_type,Block)
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
        
            async fn add(&self) -> ResultStatus {
        
                let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();
        
                let mut block = Vec::new();

                while let Ok(data) = self.data_input_receiver.recv().await {
        
                    block.push(data);
                }
        
                for transmitter in inputs_to_fill {
                    match transmitter {
                        Transmitter::$mel_output_type(sender) => {
                            sender.send(block.clone()).await.unwrap();
                            sender.close()
                        },
                        _ => panic!("{} sender expected!", std::any::type_name::<$rust_type>())
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
        
                hashmap.insert("data".to_string(), vec![Transmitter::$mel_input_type(self.data_input_sender.clone())]);
        
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

impl_StreamToBlock!(StreamU8ToBlockVecU8, "StreamU8ToBlockVecU8", u8, U8, VecU8);
impl_StreamToBlock!(StreamU16ToBlockVecU16, "StreamU16ToBlockVecU16", u16, U16, VecU16);
impl_StreamToBlock!(StreamU32ToBlockVecU32, "StreamU32ToBlockVecU32", u32, U32, VecU32);
impl_StreamToBlock!(StreamU64ToBlockVecU64, "StreamU64ToBlockVecU64", u64, U64, VecU64);
impl_StreamToBlock!(StreamU128ToBlockVecU128, "StreamU128ToBlockVecU128", u128, U128, VecU128);
impl_StreamToBlock!(StreamI8ToBlockVecI8, "StreamI8ToBlockVecI8", i8, I8, VecI8);
impl_StreamToBlock!(StreamI16ToBlockVecI16, "StreamI16ToBlockVecI16", i16, I16, VecI16);
impl_StreamToBlock!(StreamI32ToBlockVecI32, "StreamI32ToBlockVecI32", i32, I32, VecI32);
impl_StreamToBlock!(StreamI64ToBlockVecI64, "StreamI64ToBlockVecI64", i64, I64, VecI64);
impl_StreamToBlock!(StreamI128ToBlockVecI128, "StreamI128ToBlockVecI128", i128, I128, VecI128);
impl_StreamToBlock!(StreamF32ToBlockVecF32, "StreamF32ToBlockVecF32", f32, F32, VecF32);
impl_StreamToBlock!(StreamF64ToBlockVecF64, "StreamF64ToBlockVecF64", f64, F64, VecF64);
impl_StreamToBlock!(StreamBoolToBlockVecBool, "StreamBoolToBlockVecBool", bool, Bool, VecBool);
impl_StreamToBlock!(StreamByteToBlockVecByte, "StreamByteToBlockVecByte", u8, Byte, VecByte);
impl_StreamToBlock!(StreamCharToBlockVecChar, "StreamCharToBlockVecChar", char, Char, VecChar);
impl_StreamToBlock!(StreamStringToBlockVecString, "StreamStringToBlockVecString", String, String, VecString);


pub fn register(c: &mut CollectionPool) {

    c.treatments.insert(&(StreamU8ToBlockVecU8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamU16ToBlockVecU16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamU32ToBlockVecU32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamU64ToBlockVecU64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamU128ToBlockVecU128::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamI8ToBlockVecI8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamI16ToBlockVecI16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamI32ToBlockVecI32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamI64ToBlockVecI64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamI128ToBlockVecI128::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamF32ToBlockVecF32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamF64ToBlockVecF64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamBoolToBlockVecBool::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamByteToBlockVecByte::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamCharToBlockVecChar::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(StreamStringToBlockVecString::descriptor() as Arc<dyn TreatmentDescriptor>));

}

/*
    FOR DEVELOPERS

The lines above can be regenerated as will using the following script:

```
#!/bin/bash

TYPES="u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 bool byte char string"

for TYPE in $TYPES
do
    UPPER_CASE_TYPE=${TYPE^}
    echo "impl_StreamToBlock!(Stream${UPPER_CASE_TYPE}ToBlockVec${UPPER_CASE_TYPE}, \"Stream${UPPER_CASE_TYPE}ToBlockVec${UPPER_CASE_TYPE}\", $TYPE, $UPPER_CASE_TYPE, Vec$UPPER_CASE_TYPE);"
    #echo "c.treatments.insert(&(Stream${UPPER_CASE_TYPE}ToBlockVec${UPPER_CASE_TYPE}::descriptor() as Arc<dyn TreatmentDescriptor>));"

done
```
    
*/
