
use crate::core::prelude::*;

macro_rules! impl_ScalarGeneration {
    ($mod:ident, $model_mel_name:expr, $treatment_mel_name:expr, $rust_type:ty, $mel_type:ident, $send_func:ident, $send_multi_func:ident, $recv_func:ident) => {
        pub mod $mod {
            use crate::core::prelude::*;
            use std::sync::atomic::{Ordering, AtomicU64};

            #[derive(Debug)]
            struct ModelGenerator {

                world: Arc<World>,
                id: RwLock<Option<ModelId>>,

                tracks: AtomicU64,
                length: AtomicU64,
                value: RwLock<$rust_type>,

                auto_reference: RwLock<Weak<Self>>,
            }

            impl ModelGenerator {

                pub fn descriptor() -> Arc<CoreModelDescriptor> {

                    model_desc!(
                        ModelGenerator,
                        core_identifier!("generation";$model_mel_name),
                        vec![
                            parameter!("tracks", Scalar, U64, Some(Value::U64(1))),
                            parameter!("length", Scalar, U64, Some(Value::U64(1024))),
                            parameter!("value", Scalar, $mel_type, Some(Value::$mel_type(<$rust_type>::default()))),
                        ],
                        model_sources![
                            ("data";)
                        ]
                    )
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

                fn generate_data(&self, length: u64, value: $rust_type, inputs: HashMap<String, Vec<Input>>) -> Vec<TrackFuture> {

                    let future = Box::new(Box::pin(async move {

                        let data_output = Output::$mel_type(Arc::new(SendTransmitter::new()));
                        inputs.get("_data").unwrap().iter().for_each(|i| data_output.add_input(i));

                        for _ in 0..length {
                            ok_or_break!(data_output.$send_func(value.clone()).await);
                        }

                        data_output.close().await;

                        ResultStatus::Ok
                    }));

                    vec![future]
                }
            }

            impl Model for ModelGenerator {

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

                fn initialize(&self) {

                    let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
                    let future_generate = Box::pin(async move { auto_self.generate().await });

                    self.world.add_continuous_task(Box::new(future_generate));
                }

                fn shutdown(&self) {

                }

            }

            treatment!(treatment_generation,
                core_identifier!("generation";$treatment_mel_name),
                models![("generator".to_string(), super::ModelGenerator::descriptor())],
                treatment_sources![
                    (super::ModelGenerator::descriptor(), "data")
                ],
                parameters![],
                inputs![
                    input!("_data",Scalar,$mel_type,Stream)
                ],
                outputs![
                    output!("data",Scalar,$mel_type,Stream)
                ],
                host {
                    let input = host.get_input("_data");
                    let output = host.get_output("data");
                
                    while let Ok(data) = input.$recv_func().await {
                
                        ok_or_break!(output.$send_multi_func(data).await);
                    }
                
                    ResultStatus::Ok
                }
            );

            pub fn register(mut c: &mut CollectionPool) {
                c.models.insert(&(ModelGenerator::descriptor() as Arc<dyn ModelDescriptor>));
                treatment_generation::register(&mut c);
            }
        }
    };
}

impl_ScalarGeneration!(u8_generation, "ScalarU8Generator", "GenerateScalarU8", u8, U8, send_u8, send_multiple_u8, recv_u8);
impl_ScalarGeneration!(u16_generation, "ScalarU16Generator", "GenerateScalarU16", u16, U16, send_u16, send_multiple_u16, recv_u16);
impl_ScalarGeneration!(u32_generation, "ScalarU32Generator", "GenerateScalarU32", u32, U32, send_u32, send_multiple_u32, recv_u32);
impl_ScalarGeneration!(u64_generation, "ScalarU64Generator", "GenerateScalarU64", u64, U64, send_u64, send_multiple_u64, recv_u64);
impl_ScalarGeneration!(u128_generation, "ScalarU128Generator", "GenerateScalarU128", u128, U128, send_u128, send_multiple_u128, recv_u128);
impl_ScalarGeneration!(i8_generation, "ScalarI8Generator", "GenerateScalarI8", i8, I8, send_i8, send_multiple_i8, recv_i8);
impl_ScalarGeneration!(i16_generation, "ScalarI16Generator", "GenerateScalarI16", i16, I16, send_i16, send_multiple_i16, recv_i16);
impl_ScalarGeneration!(i32_generation, "ScalarI32Generator", "GenerateScalarI32", i32, I32, send_i32, send_multiple_i32, recv_i32);
impl_ScalarGeneration!(i64_generation, "ScalarI64Generator", "GenerateScalarI64", i64, I64, send_i64, send_multiple_i64, recv_i64);
impl_ScalarGeneration!(i128_generation, "ScalarI128Generator", "GenerateScalarI128", i128, I128, send_i128, send_multiple_i128, recv_i128);
impl_ScalarGeneration!(f32_generation, "ScalarF32Generator", "GenerateScalarF32", f32, F32, send_f32, send_multiple_f32, recv_f32);
impl_ScalarGeneration!(f64_generation, "ScalarF64Generator", "GenerateScalarF64", f64, F64, send_f64, send_multiple_f64, recv_f64);
impl_ScalarGeneration!(bool_generation, "ScalarBoolGenerator", "GenerateScalarBool", bool, Bool, send_bool, send_multiple_bool, recv_bool);
impl_ScalarGeneration!(byte_generation, "ScalarByteGenerator", "GenerateScalarByte", u8, Byte, send_byte, send_multiple_byte, recv_byte);
impl_ScalarGeneration!(char_generation, "ScalarCharGenerator", "GenerateScalarChar", char, Char, send_char, send_multiple_char, recv_char);
impl_ScalarGeneration!(string_generation, "ScalarStringGenerator", "GenerateScalarString", String, String, send_string, send_multiple_string, recv_string);

pub fn register(mut c: &mut CollectionPool) {

    u8_generation::register(&mut c);
    u16_generation::register(&mut c);
    u32_generation::register(&mut c);
    u64_generation::register(&mut c);
    u128_generation::register(&mut c);
    i8_generation::register(&mut c);
    i16_generation::register(&mut c);
    i32_generation::register(&mut c);
    i64_generation::register(&mut c);
    i128_generation::register(&mut c);
    f32_generation::register(&mut c);
    f64_generation::register(&mut c);
    bool_generation::register(&mut c);
    byte_generation::register(&mut c);
    char_generation::register(&mut c);
    string_generation::register(&mut c);
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
    echo "impl_ScalarGeneration!(${TYPE}_generation, \"Scalar${UPPER_CASE_TYPE}Generator\", \"GenerateScalar${UPPER_CASE_TYPE}\", $TYPE, $UPPER_CASE_TYPE, send_$TYPE, send_multiple_$TYPE, recv_$TYPE);"
    #echo "${TYPE}_generation::register(&mut c);"

done
```
    
*/
