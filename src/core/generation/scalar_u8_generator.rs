
use super::super::prelude::*;
use std::sync::atomic::{Ordering, AtomicU64, AtomicU8};

#[derive(Debug)]
pub struct ScalarU8Generator {

    world: Arc<World>,
    id: RwLock<Option<ModelId>>,

    tracks: AtomicU64,
    length: AtomicU64,
    value: AtomicU8,

    auto_reference: RwLock<Weak<Self>>,
}

impl ScalarU8Generator {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreModelDescriptor> = {

                let builder = CoreModelBuilder::new(ScalarU8Generator::new);

                let descriptor = CoreModelDescriptor::new(
                    core_identifier!("generation";"ScalarU8Generator"),
                    vec![
                        parameter!("tracks", Scalar, U64, Some(Value::U64(1))),
                        parameter!("length", Scalar, U64, Some(Value::U64(1024))),
                        parameter!("value", Scalar, U8, Some(Value::U8(0))),
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
            value: AtomicU8::new(0),

            auto_reference: RwLock::new(Weak::new()),
        });

        *model.auto_reference.write().unwrap() = Arc::downgrade(&model);

        model
    }

    pub async fn generate(&self) {

        let model_id = self.id.read().unwrap().unwrap();
        let tracks = self.tracks.load(Ordering::Relaxed);
        let length = self.length.load(Ordering::Relaxed);
        let value = self.value.load(Ordering::Relaxed);

        let generator = |inputs| {
            self.generate_data(length, value, inputs)
        };

        for _ in 0..tracks {
            self.world.create_track(model_id, "data", HashMap::new(), None, Some(&generator)).await;
        }
    }

    fn generate_data(&self, length: u64, value: u8, inputs: HashMap<String, Vec<Transmitter>>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {
            let inputs_to_fill = inputs.get("data").unwrap();

            for transmitter in inputs_to_fill {
                match transmitter {
                    Transmitter::U8(sender) => {
                        for _n in 0..length {
                            sender.send(value).await.unwrap();
                        }
                        sender.close();
                    },
                    _ => panic!("U8 sender expected!")
                }
            }

            ResultStatus::Ok
        }));

        vec![future]
    }
}

impl Model for ScalarU8Generator {

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
                    Value::U8(value) => self.value.store(*value, Ordering::Relaxed),
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
