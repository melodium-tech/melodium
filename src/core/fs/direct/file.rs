
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

    parameters.push(path_parameter);

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
    os_path: PathBuf,
    file: Option<File>,
}

impl FileModel {

    pub fn new() -> Self {
        Self {
            id: RwLock::new(None),
            path: String::new(),
            os_path: PathBuf::new(),
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

    fn set_parameter(&mut self, param: &str, value: &Value) {

        match param {
            "path" => {
                match value {
                    Value::String(path) => self.path = path.to_string(),
                    _ => panic!("Unexpected value type for 'path'."),
                }
            },
            _ => panic!("No parameter '{}' exists.", param)
        }
    }

    fn get_context_for(&self, source: &str) -> Vec<String> {

        match source {
            "read" | "write" => vec!["File".to_string()],
            _ => Vec::new(),
        }
    }

    fn initialize(&self) {

        let os_path = PathBuf::from(self.path.clone());

        let mut read = false;
        let mut write = false;

        //if os_path.is_file
    }

    fn shutdown(&self) {

    }
}
