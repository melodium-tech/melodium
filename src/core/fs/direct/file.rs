
use std::sync::{Arc, Weak, RwLock};
use async_std::path::PathBuf;
use async_std::fs::{File, OpenOptions};
use async_std::prelude::*;
use crate::executive::model::{Model, ModelId};
use crate::executive::environment::{ContextualEnvironment, GenesisEnvironment};
use crate::executive::value::Value;
use crate::logic::error::LogicError;
use crate::logic::builder::*;
use crate::logic::descriptor::{ParameterDescriptor, CoreModelDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, TreatmentDescriptor};
use crate::logic::descriptor::identifier::*;

pub fn file_descriptor() -> Arc<CoreModelDescriptor> {

    let mut parameters = Vec::new();

    let path_parameter = ParameterDescriptor::new(
        "path",
        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::String),
        None
    );

    let read_parameter = ParameterDescriptor::new(
        "read",
        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::Bool),
        Some(Value::Bool(false))
    );

    let write_parameter = ParameterDescriptor::new(
        "write",
        DataTypeDescriptor::new(DataTypeStructureDescriptor::Scalar, DataTypeTypeDescriptor::Bool),
        Some(Value::Bool(false))
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
    parameters.push(read_parameter);
    parameters.push(write_parameter);
    parameters.push(append_parameter);
    parameters.push(create_parameter);
    parameters.push(new_parameter);

    let builder = FileBuilder::new();

    let descriptor = CoreModelDescriptor::new(
        Identifier::new(Root::Core,
            vec![
                "fs".to_string(),
                "direct".to_string(),
            ],
            "File"),
        parameters,
        Box::new(builder)
    );

    let rc_descriptor = Arc::new(descriptor);
    rc_descriptor.set_autoref(&rc_descriptor);

    rc_descriptor
    
}

#[derive(Debug)]
struct FileBuilder {

}

impl FileBuilder {

    pub fn new() -> Self {
        todo!()
    }
}

impl Builder for FileBuilder {

    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        let mut file_model = FileModel::new();

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
struct FileModel {

    id: RwLock<Option<ModelId>>,

    path: String,
    read: bool,
    write: bool,
    append: bool,
    create: bool,
    new: bool,

    os_path: PathBuf,
    open_strategy: OpenOptions,
    file: Option<File>,
}

impl FileModel {

    pub fn new() -> Self {
        Self {
            id: RwLock::new(None),

            path: String::new(),
            read: false,
            write: false,
            append: false,
            create: true,
            new: false,

            os_path: PathBuf::new(),
            open_strategy: OpenOptions::new(),
            file: None,
        }
    }

    pub fn set_id(&self, id: ModelId) {
        *self.id.write().unwrap() = Some(id);
    }
}

impl Model for FileModel {
    
    fn descriptor(&self) -> Arc<CoreModelDescriptor> {
        file_descriptor()
    }

    fn set_parameter(&self, param: &str, value: &Value) {
        /*
        match param {
            "path" => {
                match value {
                    Value::String(path) => self.path = path.to_string(),
                    _ => panic!("Unexpected value type for 'path'."),
                }
            },
            "read" => {
                match value {
                    Value::Bool(read) => self.read = *read,
                    _ => panic!("Unexpected value type for 'read'."),
                }
            },
            "write" => {
                match value {
                    Value::Bool(write) => self.write = *write,
                    _ => panic!("Unexpected value type for 'write'."),
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
        */
    }

    fn get_context_for(&self, source: &str) -> Vec<String> {

        match source {
            "read" | "write" => vec!["File".to_string()],
            _ => Vec::new(),
        }
    }

    fn initialize(&self) {

        let os_path = PathBuf::from(self.path.clone());

        

        //if os_path.is_file
    }

    fn shutdown(&self) {

    }
}
