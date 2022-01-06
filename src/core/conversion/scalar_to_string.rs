
use super::super::prelude::*;

macro_rules! impl_ScalarToString {
    ($name:ident, $mel_name:expr, $input_rust_type:ty, $input_mel_type:ident) => {
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
                            core_identifier!("conversion";$mel_name),
                            models![],
                            treatment_sources![],
                            vec![],
                            vec![
                                input!("value",Scalar,$input_mel_type,Stream)
                            ],
                            vec![
                                output!("value",Scalar,String,Stream)
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
        
                    let output_string = data.to_string();
        
                    for transmitter in &inputs_to_fill {
                        match transmitter {
                            Transmitter::String(sender) => sender.send(output_string.clone()).await.unwrap(),
                            _ => panic!("{} sender expected!", std::any::type_name::<String>())
                        };
                    }
                }
        
                for transmitter in inputs_to_fill {
                    match transmitter {
                        Transmitter::String(sender) => sender.close(),
                        _ => panic!("{} sender expected!", std::any::type_name::<String>())
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
                let future = Box::new(Box::pin(async move { auto_self.stringify().await }));
        
                vec![future]
            }
            
        }
    };
}

impl_ScalarToString!(ScalarU8ToString, "ScalarU8ToString", u8, U8);
impl_ScalarToString!(ScalarU16ToString, "ScalarU16ToString", u16, U16);
impl_ScalarToString!(ScalarU32ToString, "ScalarU32ToString", u32, U32);
impl_ScalarToString!(ScalarU64ToString, "ScalarU64ToString", u64, U64);
impl_ScalarToString!(ScalarU128ToString, "ScalarU128ToString", u128, U128);
impl_ScalarToString!(ScalarI8ToString, "ScalarI8ToString", i8, I8);
impl_ScalarToString!(ScalarI16ToString, "ScalarI16ToString", i16, I16);
impl_ScalarToString!(ScalarI32ToString, "ScalarI32ToString", i32, I32);
impl_ScalarToString!(ScalarI64ToString, "ScalarI64ToString", i64, I64);
impl_ScalarToString!(ScalarI128ToString, "ScalarI128ToString", i128, I128);
impl_ScalarToString!(ScalarF32ToString, "ScalarF32ToString", f32, F32);
impl_ScalarToString!(ScalarF64ToString, "ScalarF64ToString", f64, F64);
impl_ScalarToString!(ScalarBoolToString, "ScalarBoolToString", bool, Bool);
impl_ScalarToString!(ScalarByteToString, "ScalarByteToString", u8, Byte);
impl_ScalarToString!(ScalarCharToString, "ScalarCharToString", char, Char);


