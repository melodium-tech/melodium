
use crate::core::prelude::*;

macro_rules! impl_BlockToStream {
    ($name:ident, $mel_name:expr, $rust_type:ty, $mel_scalar_type:ident, $mel_vector_type:ident) => {
        struct $name {

            world: Arc<World>,
        
            data_output_transmitters: RwLock<Vec<Transmitter>>,
            data_input_sender: Sender<Vec<$rust_type>>,
            data_input_receiver: Receiver<Vec<$rust_type>>,
        
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
                                input!("data",Vector,$mel_scalar_type,Block)
                            ],
                            vec![
                                output!("data",Scalar,$mel_scalar_type,Stream)
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

                if let Ok(data) = self.data_input_receiver.recv().await {

                    for transmitter in &inputs_to_fill {
                        match transmitter {
                            Transmitter::$mel_scalar_type(sender) => for item in &data { sender.send(item.clone()).await.unwrap() },
                            _ => panic!("{} sender expected!", std::any::type_name::<$rust_type>())
                        };
                    }
                }

                for transmitter in inputs_to_fill {
                    match transmitter {
                        Transmitter::$mel_scalar_type(sender) => sender.close(),
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
        
                hashmap.insert("data".to_string(), vec![Transmitter::$mel_vector_type(self.data_input_sender.clone())]);
        
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

impl_BlockToStream!(BlockVecU8ToStreamU8, "BlockVecU8ToStreamU8", u8, U8, VecU8);
impl_BlockToStream!(BlockVecU16ToStreamU16, "BlockVecU16ToStreamU16", u16, U16, VecU16);
impl_BlockToStream!(BlockVecU32ToStreamU32, "BlockVecU32ToStreamU32", u32, U32, VecU32);
impl_BlockToStream!(BlockVecU64ToStreamU64, "BlockVecU64ToStreamU64", u64, U64, VecU64);
impl_BlockToStream!(BlockVecU128ToStreamU128, "BlockVecU128ToStreamU128", u128, U128, VecU128);
impl_BlockToStream!(BlockVecI8ToStreamI8, "BlockVecI8ToStreamI8", i8, I8, VecI8);
impl_BlockToStream!(BlockVecI16ToStreamI16, "BlockVecI16ToStreamI16", i16, I16, VecI16);
impl_BlockToStream!(BlockVecI32ToStreamI32, "BlockVecI32ToStreamI32", i32, I32, VecI32);
impl_BlockToStream!(BlockVecI64ToStreamI64, "BlockVecI64ToStreamI64", i64, I64, VecI64);
impl_BlockToStream!(BlockVecI128ToStreamI128, "BlockVecI128ToStreamI128", i128, I128, VecI128);
impl_BlockToStream!(BlockVecF32ToStreamF32, "BlockVecF32ToStreamF32", f32, F32, VecF32);
impl_BlockToStream!(BlockVecF64ToStreamF64, "BlockVecF64ToStreamF64", f64, F64, VecF64);
impl_BlockToStream!(BlockVecBoolToStreamBool, "BlockVecBoolToStreamBool", bool, Bool, VecBool);
impl_BlockToStream!(BlockVecByteToStreamByte, "BlockVecByteToStreamByte", u8, Byte, VecByte);
impl_BlockToStream!(BlockVecCharToStreamChar, "BlockVecCharToStreamChar", char, Char, VecChar);
impl_BlockToStream!(BlockVecStringToStreamString, "BlockVecStringToStreamString", String, String, VecString);

pub fn register(c: &mut CollectionPool) {

    c.treatments.insert(&(BlockVecU8ToStreamU8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecU16ToStreamU16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecU32ToStreamU32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecU64ToStreamU64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecU128ToStreamU128::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecI8ToStreamI8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecI16ToStreamI16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecI32ToStreamI32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecI64ToStreamI64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecI128ToStreamI128::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecF32ToStreamF32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecF64ToStreamF64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecBoolToStreamBool::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecByteToStreamByte::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecCharToStreamChar::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(BlockVecStringToStreamString::descriptor() as Arc<dyn TreatmentDescriptor>));

}
