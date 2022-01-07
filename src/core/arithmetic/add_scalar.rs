
use super::super::prelude::*;

macro_rules! impl_AddScalar {
    ($name:ident, $mel_name:expr, $rust_type:ty, $mel_type:ident) => {
        struct $name {

            world: Arc<World>,
        
            value: RwLock<$rust_type>,
        
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
                            core_identifier!("arithmetic";$mel_name),
                            models![],
                            treatment_sources![],
                            vec![
                                parameter!("value",Scalar,$mel_type,Some(crate::executive::value::Value::$mel_type(<$rust_type>::default())))
                            ],
                            vec![
                                input!("data",Scalar,$mel_type,Stream)
                            ],
                            vec![
                                output!("data",Scalar,$mel_type,Stream)
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
                    value: RwLock::new(<$rust_type>::default()),
                    data_output_transmitters: RwLock::new(Vec::new()),
                    data_input_sender: data_input.0,
                    data_input_receiver: data_input.1,
                    auto_reference: RwLock::new(Weak::new()),
                });
        
                *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);
        
                treatment
            }
        
            async fn add(&self) -> ResultStatus {
        
                let value = *self.value.read().unwrap();
                let inputs_to_fill = self.data_output_transmitters.read().unwrap().clone();
        
                while let Ok(data) = self.data_input_receiver.recv().await {
        
                    let output_data = data + value;
        
                    for transmitter in &inputs_to_fill {
                        match transmitter {
                            Transmitter::$mel_type(sender) => sender.send(output_data).await.unwrap(),
                            _ => panic!("{} sender expected!", std::any::type_name::<$rust_type>())
                        };
                    }
                }
        
                for transmitter in inputs_to_fill {
                    match transmitter {
                        Transmitter::$mel_type(sender) => sender.close(),
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
                
                match param {
                    "value" => {
                        match value {
                            Value::$mel_type(value) => *self.value.write().unwrap() = *value,
                            _ => panic!("Unexpected value type for 'value'."),
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
                    "data" => self.data_output_transmitters.write().unwrap().extend(transmitter),
                    _ => panic!("No output '{}' exists.", output_name)
                }
            }
        
            fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {
        
                let mut hashmap = HashMap::new();
        
                hashmap.insert("data".to_string(), vec![Transmitter::$mel_type(self.data_input_sender.clone())]);
        
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

impl_AddScalar!(AddScalarI8, "AddScalarI8", i8, I8);
impl_AddScalar!(AddScalarI16, "AddScalarI16", i16, I16);
impl_AddScalar!(AddScalarI32, "AddScalarI32", i32, I32);
impl_AddScalar!(AddScalarI64, "AddScalarI64", i64, I64);
impl_AddScalar!(AddScalarI128, "AddScalarI128", i128, I128);

impl_AddScalar!(AddScalarU8, "AddScalarU8", u8, U8);
impl_AddScalar!(AddScalarU16, "AddScalarU16", u16, U16);
impl_AddScalar!(AddScalarU32, "AddScalarU32", u32, U32);
impl_AddScalar!(AddScalarU64, "AddScalarU64", u64, U64);
impl_AddScalar!(AddScalarU128, "AddScalarU128", u128, U128);

impl_AddScalar!(AddScalarF32, "AddScalarF32", f32, F32);
impl_AddScalar!(AddScalarF64, "AddScalarF64", f64, F64);

pub fn register(c: &mut CollectionPool) {

    c.treatments.insert(&(AddScalarI8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(AddScalarI16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(AddScalarI32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(AddScalarI64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(AddScalarI128::descriptor() as Arc<dyn TreatmentDescriptor>));

    c.treatments.insert(&(AddScalarU8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(AddScalarU16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(AddScalarU32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(AddScalarU64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(AddScalarU128::descriptor() as Arc<dyn TreatmentDescriptor>));

    c.treatments.insert(&(AddScalarF32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.treatments.insert(&(AddScalarF64::descriptor() as Arc<dyn TreatmentDescriptor>));
}

