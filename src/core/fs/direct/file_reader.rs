
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use async_std::path::PathBuf;
use async_std::fs::{File, OpenOptions};
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
use crate::logic::descriptor::{ParameterDescriptor, CoreModelDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, TreatmentDescriptor};
use crate::logic::descriptor::identifier::*;

#[derive(Debug)]
pub struct FileReaderModel {

    world: Arc<World>,
    id: RwLock<Option<ModelId>>,

    path: RwLock<String>,

    auto_reference: RwLock<Weak<Self>>,
}

impl FileReaderModel {

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        let model = Arc::new(Self {
            world,
            id: RwLock::new(None),

            path: RwLock::new(String::new()),

            auto_reference: RwLock::new(Weak::new()),
        });

        *model.auto_reference.write().unwrap() = Arc::downgrade(&model);

        model
    }

    pub fn path(&self) -> String {
        self.path.read().unwrap().clone()
    }

    async fn read(&self) {

        let os_path = PathBuf::from(self.path());
        let open_result = File::open(&os_path).await;

        if let Ok(file) = open_result {

            let mut file_context = Context::new();

            let path = if let Ok(os_string) = os_path.canonicalize().await {
                os_string.into_os_string().into_string().unwrap_or_default()
            } else { "".to_string() };
            file_context.set_value("path", Value::String(path));

            let directory = if let Some(path) = os_path.parent() {
                if let Some(path) = path.to_str() {
                    path.to_string()
                }
                else { "".to_string() }
            }
            else { "".to_string() };
            file_context.set_value("directory", Value::String(directory));

            let name = if let Some(name) = os_path.file_name() {
                if let Some(name) = name.to_str() {
                    name.to_string()
                }
                else { "".to_string() }
            }
            else { "".to_string() };
            file_context.set_value("name", Value::String(name));

            let stem = if let Some(stem) = os_path.file_stem() {
                if let Some(stem) = stem.to_str() {
                    stem.to_string()
                }
                else { "".to_string() }
            }
            else { "".to_string() };
            file_context.set_value("stem", Value::String(stem));

            let extension = if let Some(extension) = os_path.file_stem() {
                if let Some(extension) = extension.to_str() {
                    extension.to_string()
                }
                else { "".to_string() }
            }
            else { "".to_string() };
            file_context.set_value("extension", Value::String(extension));

            let mut contextes = HashMap::new();
            contextes.insert("File".to_string(), file_context);

            let inputs = self.world.create_track(self.id.read().unwrap().unwrap(), "read", contextes, None);
            let inputs_to_fill = inputs.get("data").unwrap();

            let mut bytes = file.bytes();
            while let Some(possible_byte) = bytes.next().await {

                let byte = possible_byte.unwrap();

                for transmitter in inputs_to_fill {
                    match transmitter {
                        Transmitter::Byte(sender) => sender.send(byte).await.unwrap(),
                        _ => panic!("Byte sender expected!")
                    }
                }
            }
        }

        // Todo manage failures
    }
}

impl Model for FileReaderModel {
    
    fn descriptor(&self) -> &Arc<CoreModelDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreModelDescriptor> = {
                let mut parameters = Vec::new();

                let path_parameter = ParameterDescriptor::new(
                    "path",
                    DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::String),
                    None
                );

                parameters.push(path_parameter);

                let builder = CoreModelBuilder::new(FileReaderModel::new);

                let descriptor = CoreModelDescriptor::new(
                    Identifier::new(Root::Core,
                        vec![
                            "fs".to_string(),
                            "direct".to_string(),
                        ],
                        "FileReader"),
                    parameters,
                    Box::new(builder)
                );

                let rc_descriptor = Arc::new(descriptor);
                rc_descriptor.set_autoref(&rc_descriptor);

                rc_descriptor
            };
        }
        
        &DESCRIPTOR
    }

    fn set_id(&self, id: ModelId) {
        *self.id.write().unwrap() = Some(id);
    }

    fn set_parameter(&self, param: &str, value: &Value) {

        match param {
            "path" => {
                match value {
                    Value::String(path) => *self.path.write().unwrap() = path.to_string(),
                    _ => panic!("Unexpected value type for 'path'."),
                }
            },
            _ => panic!("No parameter '{}' exists.", param)
        }
    }

    fn get_context_for(&self, source: &str) -> Vec<String> {

        match source {
            "read" => vec!["File".to_string()],
            _ => Vec::new(),
        }
    }

    fn initialize(&self) {

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future_read = async move { auto_self.read().await };

        self.world.add_continuous_task(Box::new(future_read));
    }

    fn shutdown(&self) {

    }
}
