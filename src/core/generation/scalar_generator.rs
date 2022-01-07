
use crate::core::prelude::*;
use std::sync::atomic::{Ordering, AtomicU64};

macro_rules! impl_ScalarGeneration {
    ($model_name:ident, $model_mel_name:expr, $treatment_name:ident, $treatment_mel_name:expr, $rust_type:ty, $mel_type:ident) => {

        #[derive(Debug)]
        struct $model_name {

            world: Arc<World>,
            id: RwLock<Option<ModelId>>,

            tracks: AtomicU64,
            length: AtomicU64,
            value: RwLock<$rust_type>,

            auto_reference: RwLock<Weak<Self>>,
        }

        impl $model_name {

            pub fn descriptor() -> Arc<CoreModelDescriptor> {

                lazy_static! {
                    static ref DESCRIPTOR: Arc<CoreModelDescriptor> = {

                        let builder = CoreModelBuilder::new($model_name::new);

                        let descriptor = CoreModelDescriptor::new(
                            core_identifier!("generation";$model_mel_name),
                            vec![
                                parameter!("tracks", Scalar, U64, Some(Value::U64(1))),
                                parameter!("length", Scalar, U64, Some(Value::U64(1024))),
                                parameter!("value", Scalar, $mel_type, Some(Value::$mel_type(<$rust_type>::default()))),
                            ],
                            model_sources![
                                ("data";)
                            ],
                            Box::new(builder)
                        );

                        let rc_descriptor = Arc::new(descriptor);
                        rc_descriptor.set_autoref(&rc_descriptor);

                        rc_descriptor
                    };
                }
                
                Arc::clone(&DESCRIPTOR)
            }

            pub fn new(world: Arc<World>) -> Arc<dyn Model> {

                let model = Arc::new(Self {
                    world,
                    id: RwLock::new(None),

                    tracks: AtomicU64::new(1),
                    length: AtomicU64::new(1024),
                    value: RwLock::new(<$rust_type>::default()),

                    auto_reference: RwLock::new(Weak::new()),
                });

                *model.auto_reference.write().unwrap() = Arc::downgrade(&model);

                model
            }

            pub async fn generate(&self) {

                let model_id = self.id.read().unwrap().unwrap();
                let tracks = self.tracks.load(Ordering::Relaxed);
                let length = self.length.load(Ordering::Relaxed);

                let generator = |inputs| {
                    self.generate_data(
                        length,
                        self.value.read().unwrap().clone(),
                        inputs
                    )
                };

                for _ in 0..tracks {
                    self.world.create_track(model_id, "data", HashMap::new(), None, Some(&generator)).await;
                }
            }

            fn generate_data(&self, length: u64, value: $rust_type, inputs: HashMap<String, Vec<Transmitter>>) -> Vec<TrackFuture> {

                let future = Box::new(Box::pin(async move {
                    let inputs_to_fill = inputs.get("data").unwrap();

                    for transmitter in inputs_to_fill {
                        match transmitter {
                            Transmitter::$mel_type(sender) => {
                                for _n in 0..length {
                                    sender.send(value.clone()).await.unwrap();
                                }
                                sender.close();
                            },
                            _ => panic!("{} sender expected!", std::any::type_name::<$rust_type>())
                        }
                    }

                    ResultStatus::Ok
                }));

                vec![future]
            }
        }

        impl Model for $model_name {

            fn descriptor(&self) -> Arc<CoreModelDescriptor> {
                Self::descriptor()
            }

            fn id(&self) -> Option<ModelId> {
                *self.id.read().unwrap()
            }

            fn set_id(&self, id: ModelId) {
                *self.id.write().unwrap() = Some(id);
            }

            fn set_parameter(&self, param: &str, value: &Value) {

                match param {
                    "tracks" => {
                        match value {
                            Value::U64(tracks) => self.tracks.store(*tracks, Ordering::Relaxed),
                            _ => panic!("Unexpected value type for 'tracks'."),
                        }
                    },
                    "length" => {
                        match value {
                            Value::U64(length) => self.length.store(*length, Ordering::Relaxed),
                            _ => panic!("Unexpected value type for 'length'."),
                        }
                    },
                    "value" => {
                        match value {
                            Value::$mel_type(value) => *self.value.write().unwrap() = value.clone(),
                            _ => panic!("Unexpected value type for 'value'."),
                        }
                    },
                    _ => panic!("No parameter '{}' exists.", param)
                }
            }

            fn get_context_for(&self, source: &str) -> Vec<String> {

                Vec::new()
            }

            fn initialize(&self) {

                let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
                let future_generate = Box::pin(async move { auto_self.generate().await });

                self.world.add_continuous_task(Box::new(future_generate));
            }

            fn shutdown(&self) {

            }

        }

        struct $treatment_name {

            world: Arc<World>,
        
            generator: RwLock<Option<Arc<$model_name>>>,
            data_transmitters: RwLock<Vec<Transmitter>>,
        
            auto_reference: RwLock<Weak<Self>>,
        }
        
        impl $treatment_name {
        
            pub fn descriptor() -> Arc<CoreTreatmentDescriptor> {
        
                lazy_static! {
                    static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {
        
                        let rc_descriptor = CoreTreatmentDescriptor::new(
                            core_identifier!("generation";$treatment_mel_name),
                            vec![("generator".to_string(), $model_name::descriptor())],
                            treatment_sources![
                                ($model_name::descriptor(), "data")
                            ],
                            vec![],
                            vec![],
                            vec![
                                output!("data", Scalar, $mel_type, Stream)
                            ],
                            $treatment_name::new,
                        );
        
                        rc_descriptor
                    };
                }
        
                Arc::clone(&DESCRIPTOR)
            }
        
            pub fn new(world: Arc<World>) -> Arc<dyn Treatment> {
                let treatment = Arc::new(Self {
                    world,
                    generator: RwLock::new(None),
                    data_transmitters: RwLock::new(Vec::new()),
                    auto_reference: RwLock::new(Weak::new()),
                });
        
                *treatment.auto_reference.write().unwrap() = Arc::downgrade(&treatment);
        
                treatment
            }
        }
        
        impl Treatment for $treatment_name {
        
            fn descriptor(&self) -> Arc<CoreTreatmentDescriptor> {
                Self::descriptor()
            }
        
            fn set_parameter(&self, param: &str, value: &Value) {
                panic!("No parameter expected.")
            }
        
            fn set_model(&self, name: &str, model: &Arc<dyn Model>) {
        
                match name {
                    "generator" => *self.generator.write().unwrap() = Some(Arc::clone(&model).downcast_arc::<$model_name>().unwrap()),
                    _ => panic!("No model '{}' expected.", name)
                }
            }
        
            fn set_output(&self, output_name: &str, transmitter: Vec<Transmitter>) {
                
                match output_name {
                    "data" => self.data_transmitters.write().unwrap().extend(transmitter),
                    _ => panic!("No output '{}' exists.", output_name)
                }
            }
        
            fn get_inputs(&self) -> HashMap<String, Vec<Transmitter>> {
        
                let mut hashmap = HashMap::new();
        
                hashmap.insert("data".to_string(), self.data_transmitters.read().unwrap().clone());
        
                hashmap
            }
        
            fn prepare(&self) -> Vec<TrackFuture> {
                Vec::new()
            }
            
        }
    };
}

impl_ScalarGeneration!(ScalarU8Generator, "ScalarU8Generator", GenerateScalarU8, "GenerateScalarU8", u8, U8);
impl_ScalarGeneration!(ScalarU16Generator, "ScalarU16Generator", GenerateScalarU16, "GenerateScalarU16", u16, U16);
impl_ScalarGeneration!(ScalarU32Generator, "ScalarU32Generator", GenerateScalarU32, "GenerateScalarU32", u32, U32);
impl_ScalarGeneration!(ScalarU64Generator, "ScalarU64Generator", GenerateScalarU64, "GenerateScalarU64", u64, U64);
impl_ScalarGeneration!(ScalarU128Generator, "ScalarU128Generator", GenerateScalarU128, "GenerateScalarU128", u128, U128);
impl_ScalarGeneration!(ScalarI8Generator, "ScalarI8Generator", GenerateScalarI8, "GenerateScalarI8", i8, I8);
impl_ScalarGeneration!(ScalarI16Generator, "ScalarI16Generator", GenerateScalarI16, "GenerateScalarI16", i16, I16);
impl_ScalarGeneration!(ScalarI32Generator, "ScalarI32Generator", GenerateScalarI32, "GenerateScalarI32", i32, I32);
impl_ScalarGeneration!(ScalarI64Generator, "ScalarI64Generator", GenerateScalarI64, "GenerateScalarI64", i64, I64);
impl_ScalarGeneration!(ScalarI128Generator, "ScalarI128Generator", GenerateScalarI128, "GenerateScalarI128", i128, I128);
impl_ScalarGeneration!(ScalarF32Generator, "ScalarF32Generator", GenerateScalarF32, "GenerateScalarF32", f32, F32);
impl_ScalarGeneration!(ScalarF64Generator, "ScalarF64Generator", GenerateScalarF64, "GenerateScalarF64", f64, F64);
impl_ScalarGeneration!(ScalarBoolGenerator, "ScalarBoolGenerator", GenerateScalarBool, "GenerateScalarBool", bool, Bool);
impl_ScalarGeneration!(ScalarByteGenerator, "ScalarByteGenerator", GenerateScalarByte, "GenerateScalarByte", u8, Byte);
impl_ScalarGeneration!(ScalarCharGenerator, "ScalarCharGenerator", GenerateScalarChar, "GenerateScalarChar", char, Char);
impl_ScalarGeneration!(ScalarStringGenerator, "ScalarStringGenerator", GenerateScalarString, "GenerateScalarString", String, String);

pub fn register(c: &mut CollectionPool) {

    c.models.insert(&(ScalarU8Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarU8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarU16Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarU16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarU32Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarU32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarU64Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarU64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarU128Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarU128::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarI8Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarI8::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarI16Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarI16::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarI32Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarI32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarI64Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarI64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarI128Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarI128::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarF32Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarF32::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarF64Generator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarF64::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarBoolGenerator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarBool::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarByteGenerator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarByte::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarCharGenerator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarChar::descriptor() as Arc<dyn TreatmentDescriptor>));
    c.models.insert(&(ScalarStringGenerator::descriptor() as Arc<dyn ModelDescriptor>));
    c.treatments.insert(&(GenerateScalarString::descriptor() as Arc<dyn TreatmentDescriptor>));

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
    #echo "impl_ScalarGeneration!(Scalar${UPPER_CASE_TYPE}Generator, \"Scalar${UPPER_CASE_TYPE}Generator\", GenerateScalar${UPPER_CASE_TYPE}, \"GenerateScalar${UPPER_CASE_TYPE}\", $TYPE, $UPPER_CASE_TYPE);"
    echo "c.models.insert(&(Scalar${UPPER_CASE_TYPE}Generator::descriptor() as Arc<dyn ModelDescriptor>));"
    echo "c.treatments.insert(&(GenerateScalar${UPPER_CASE_TYPE}::descriptor() as Arc<dyn TreatmentDescriptor>));"

done
```
    
*/
