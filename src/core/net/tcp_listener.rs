

use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use async_std::net::*;
use async_std::task::block_on;
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
pub struct TcpListenerModel {

    world: Arc<World>,
    id: RwLock<Option<ModelId>>,

    socket_address: RwLock<String>,

    auto_reference: RwLock<Weak<Self>>,
}

impl TcpListenerModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreModelDescriptor> = {
                let mut parameters = Vec::new();

                let socket_address_parameter = ParameterDescriptor::new(
                    "socket_address",
                    DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::String),
                    None
                );

                parameters.push(socket_address_parameter);

                let mut sources = HashMap::new();

                sources.insert("connection".to_string(), vec![Arc::clone(Contexts::get("TcpConnection").unwrap())]);

                let builder = CoreModelBuilder::new(TcpListenerModel::new);

                let descriptor = CoreModelDescriptor::new(
                    Identifier::new(Root::Core,
                        vec![
                            "net".to_string(),
                        ],
                        "TcpListener"),
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

            socket_address: RwLock::new(String::new()),

            auto_reference: RwLock::new(Weak::new()),
        });

        *model.auto_reference.write().unwrap() = Arc::downgrade(&model);

        model
    }

    pub fn socket_address(&self) -> String {
        self.socket_address.read().unwrap().clone()
    }

    async fn listen(&self) {

        // Todo manage io error
        if let Ok(listener) = TcpListener::bind(self.socket_address()).await {

            while let Ok((stream, addr)) = listener.accept().await {


                let mut tcp_connection_context = Context::new();

                let mut contextes = HashMap::new();
                contextes.insert("TcpConnection".to_string(), tcp_connection_context);

                let model_id = self.id.read().unwrap().unwrap();
                let inputs = self.world.create_track(model_id, "connection", contextes, None).await;
                let inputs_to_fill = inputs.get("data").unwrap();
            }
        }

        

        // Todo manage failures
    }
}

impl Model for TcpListenerModel {
    
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
            "socket_address" => {
                match value {
                    Value::String(path) => *self.socket_address.write().unwrap() = path.to_string(),
                    _ => panic!("Unexpected value type for 'socket_address'."),
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
        let continuous_future = Box::pin(async move { auto_self.listen().await });

        self.world.add_continuous_task(Box::new(continuous_future));
    }

    fn shutdown(&self) {

    }
}
