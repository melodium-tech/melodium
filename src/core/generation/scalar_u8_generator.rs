
use futures::future::join_all;
use std::sync::atomic::{Ordering, AtomicU64, AtomicU8};
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use async_std::prelude::*;
use crate::executive::model::{Model, ModelId};
use crate::executive::world::World;
use crate::executive::environment::{ContextualEnvironment, GenesisEnvironment};
use crate::executive::context::Context;
use crate::executive::value::Value;
use crate::executive::transmitter::Transmitter;
use crate::logic::error::LogicError;
use crate::logic::builder::*;
use crate::logic::contexts::Contexts;
use crate::logic::descriptor::{ParameterDescriptor, CoreModelDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, TreatmentDescriptor};
use crate::logic::descriptor::identifier::*;

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
                let mut parameters = Vec::new();

                let tracks_parameter = ParameterDescriptor::new(
                    "tracks",
                    DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::U64),
                    Some(Value::U64(1))
                );

                parameters.push(tracks_parameter);

                let length_parameter = ParameterDescriptor::new(
                    "length",
                    DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::U64),
                    Some(Value::U64(1024))
                );

                parameters.push(length_parameter);

                let value_parameter = ParameterDescriptor::new(
                    "value",
                    DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::U8),
                    Some(Value::U8(0))
                );

                parameters.push(value_parameter);

                let mut sources = HashMap::new();

                sources.insert("data".to_string(), vec![]);

                let builder = CoreModelBuilder::new(ScalarU8Generator::new);

                let descriptor = CoreModelDescriptor::new(
                    Identifier::new(Root::Core,
                        vec![
                            "generation".to_string(),
                        ],
                        "ScalarU8Generator"),
                    parameters,
                    sources,
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

        let mut generators = Vec::new();
        for _ in 0..tracks {
            generators.push(self.generate_track(model_id, length, value));
        }

        join_all(generators).await;
    }

    async fn generate_track(&self, id: u64, length: u64, value: u8) {
        
        let inputs = self.world.create_track(id, "data", HashMap::new(), None).await;
        let inputs_to_fill = inputs.get("data").unwrap();

        for transmitter in inputs_to_fill {
            match transmitter {
                Transmitter::U8(sender) => {
                    for _ in 0..length  {
                        sender.send(value).await.unwrap();
                    }
                    sender.close();
                },
                _ => panic!("U8 sender expected!")
            }
        }
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
