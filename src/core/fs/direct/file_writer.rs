
use std::sync::{Arc, Weak, RwLock};
use async_std::path::PathBuf;
use async_std::fs::{File, OpenOptions};
use async_std::task::block_on;
use async_std::prelude::*;
use crate::executive::model::{Model, ModelId};
use crate::executive::world::World;
use crate::executive::environment::{ContextualEnvironment, GenesisEnvironment};
use crate::executive::value::Value;
use crate::logic::error::LogicError;
use crate::logic::builder::*;
use crate::logic::descriptor::{ParameterDescriptor, CoreModelDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, TreatmentDescriptor};
use crate::logic::descriptor::identifier::*;

pub fn file_writer_descriptor() -> Arc<CoreModelDescriptor> {

    let mut parameters = Vec::new();

    let path_parameter = ParameterDescriptor::new(
        "path",
        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::String),
        None
    );

    let append_parameter = ParameterDescriptor::new(
        "append",
        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::Bool),
        Some(Value::Bool(false))
    );

    let create_parameter = ParameterDescriptor::new(
        "create",
        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::Bool),
        Some(Value::Bool(true))
    );

    let new_parameter = ParameterDescriptor::new(
        "new",
        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::Bool),
        Some(Value::Bool(false))
    );

    parameters.push(path_parameter);
    parameters.push(append_parameter);
    parameters.push(create_parameter);
    parameters.push(new_parameter);

    let builder = FileWriterBuilder::new();

    let descriptor = CoreModelDescriptor::new(
        Identifier::new(Root::Core,
            vec![
                "fs".to_string(),
                "direct".to_string(),
            ],
            "FileWriter"),
        parameters,
        Box::new(builder)
    );

    let rc_descriptor = Arc::new(descriptor);
    rc_descriptor.set_autoref(&rc_descriptor);

    rc_descriptor
    
}

#[derive(Debug)]
struct FileWriterBuilder {

}

impl FileWriterBuilder {

    pub fn new() -> Self {
        todo!()
    }
}

impl Builder for FileWriterBuilder {

    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        let mut file_model = FileWriterModel::new(environment.world());

        for (name, value) in environment.variables() {
            file_model.set_parameter(name, value);
        }

        let rc_model = Arc::new(file_model);

        let id = environment.register_model(Arc::clone(&rc_model) as Arc<dyn Model>);

        rc_model.set_id(id);
        
        Ok(StaticBuildResult::Model(rc_model))
    }

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {
        None
    }

    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {
        None
    }

    fn check_dynamic_build(&self, build: BuildId, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {
        None
    }

    fn check_give_next(&self, within_build: BuildId, for_label: String, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {
        None
    }
}

#[derive(Debug)]
struct FileWriterModel {

    world: Arc<World>,
    id: RwLock<Option<ModelId>>,

    path: String,
    append: bool,
    create: bool,
    new: bool,

    os_path: RwLock<PathBuf>,
    open_strategy: RwLock<OpenOptions>,
    file: RwLock<Option<File>>,
}

impl FileWriterModel {

    pub fn new(world: Arc<World>) -> Self {
        Self {
            world,
            id: RwLock::new(None),

            path: String::new(),
            append: false,
            create: true,
            new: false,

            os_path: RwLock::new(PathBuf::new()),
            open_strategy: RwLock::new(OpenOptions::new()),
            file: RwLock::new(None),
        }
    }

    pub fn set_id(&self, id: ModelId) {
        *self.id.write().unwrap() = Some(id);
    }
}

impl Model for FileWriterModel {
    
    fn descriptor(&self) -> Arc<CoreModelDescriptor> {
        file_writer_descriptor()
    }

    fn set_parameter(&mut self, param: &str, value: &Value) {

        match param {
            "path" => {
                match value {
                    Value::String(path) => self.path = path.to_string(),
                    _ => panic!("Unexpected value type for 'path'."),
                }
            },
            "append" => {
                match value {
                    Value::Bool(append) => self.append = *append,
                    _ => panic!("Unexpected value type for 'append'."),
                }
            },
            "create" => {
                match value {
                    Value::Bool(create) => self.create = *create,
                    _ => panic!("Unexpected value type for 'create'."),
                }
            },
            "new" => {
                match value {
                    Value::Bool(new) => self.new = *new,
                    _ => panic!("Unexpected value type for 'new'."),
                }
            },
            _ => panic!("No parameter '{}' exists.", param)
        }
    }

    fn get_context_for(&self, source: &str) -> Vec<String> {

        Vec::new()
    }

    fn initialize(&self) {

        let os_path = PathBuf::from(self.path.clone());

        *self.os_path.write().unwrap() = os_path;

        self.open_strategy.write().unwrap().write(true)
            .append(self.append)
            .truncate(!self.append)
            .create(self.create)
            .create_new(self.new);

        // See where to enable reading itself
        // probably register something inside the World.
    }

    fn shutdown(&self) {

        if let Some(file) = &*self.file.read().unwrap() {
            let result = block_on(file.sync_all());

            if result.is_err() {
                panic!("FileWriter #{} sync_all error '{}'", self.id.read().unwrap().unwrap(), result.unwrap_err())
            }
        }
    }
}
