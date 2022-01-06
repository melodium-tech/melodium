
use super::super::prelude::*;

macro_rules! impl_VectorToString {
    ($name:ident, $mel_name:expr, $input_rust_type:ty, $input_mel_type:ident, $input_trans_type:ident) => {
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
                            core_identifier!("conversion";$mel_name),
                            models![],
                            treatment_sources![],
                            vec![],
                            vec![
                                input!("value",Vector,$input_mel_type,Stream)
                            ],
                            vec![
                                output!("value",Vector,String,Stream)
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
        
            async fn stringify(&self) -> ResultStatus {
        
                let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();
        
                while let Ok(data) = self.data_input_receiver.recv().await {
        
                    let output_strings: Vec<String> = data.iter().map(|v| v.to_string()).collect();
        
                    for transmitter in &inputs_to_fill {
                        match transmitter {
                            Transmitter::VecString(sender) => sender.send(output_strings.clone()).await.unwrap(),
                            _ => panic!("{} sender expected!", std::any::type_name::<Vec<String>>())
                        };
                    }
                }
        
                for transmitter in inputs_to_fill {
                    match transmitter {
                        Transmitter::String(sender) => sender.close(),
                        _ => panic!("{} sender expected!", std::any::type_name::<Vec<String>>())
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
                let future = Box::new(Box::pin(async move { auto_self.stringify().await }));
        
                vec![future]
            }
            
        }
    };
}

impl_VectorToString!(VectorU8ToVectorString, "VectorU8ToVectorString", u8, U8, VecU8);
impl_VectorToString!(VectorU16ToVectorString, "VectorU16ToVectorString", u16, U16, VecU16);
impl_VectorToString!(VectorU32ToVectorString, "VectorU32ToVectorString", u32, U32, VecU32);
impl_VectorToString!(VectorU64ToVectorString, "VectorU64ToVectorString", u64, U64, VecU64);
impl_VectorToString!(VectorU128ToVectorString, "VectorU128ToVectorString", u128, U128, VecU128);
impl_VectorToString!(VectorI8ToVectorString, "VectorI8ToVectorString", i8, I8, VecI8);
impl_VectorToString!(VectorI16ToVectorString, "VectorI16ToVectorString", i16, I16, VecI16);
impl_VectorToString!(VectorI32ToVectorString, "VectorI32ToVectorString", i32, I32, VecI32);
impl_VectorToString!(VectorI64ToVectorString, "VectorI64ToVectorString", i64, I64, VecI64);
impl_VectorToString!(VectorI128ToVectorString, "VectorI128ToVectorString", i128, I128, VecI128);
impl_VectorToString!(VectorF32ToVectorString, "VectorF32ToVectorString", f32, F32, VecF32);
impl_VectorToString!(VectorF64ToVectorString, "VectorF64ToVectorString", f64, F64, VecF64);
impl_VectorToString!(VectorBoolToVectorString, "VectorBoolToVectorString", bool, Bool, VecBool);
impl_VectorToString!(VectorByteToVectorString, "VectorByteToVectorString", u8, Byte, VecByte);
impl_VectorToString!(VectorCharToVectorString, "VectorCharToVectorString", char, Char, VecChar);



