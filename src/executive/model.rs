
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use downcast_rs::{DowncastSync, impl_downcast};
use crate::executive::world::World;
use super::value::Value;
use super::super::logic::descriptor::CoreModelDescriptor;
use crate::logic::descriptor::parameterized::Parameterized;

pub type ModelId = u64;

pub trait Model : Debug + DowncastSync + Send + Sync {

    fn descriptor(&self) -> Arc<CoreModelDescriptor>;

    fn id(&self) -> Option<ModelId>;
    fn set_id(&self, id: ModelId);

    fn set_parameter(&self, param: &str, value: &Value);

    fn initialize(&self);
    fn shutdown(&self);
}
impl_downcast!(sync Model);

#[derive(Debug)]
pub struct ModelHelper {

    descriptor: Arc<CoreModelDescriptor>,
    world: Arc<World>,
    id: Mutex<Option<ModelId>>,
    parameters: Mutex<HashMap<String, Value>>,
}

impl ModelHelper {

    pub fn new(descriptor: Arc<CoreModelDescriptor>, world: Arc<World>) -> Self {
        Self {
            descriptor,
            world,
            id: Mutex::new(None),
            parameters: Mutex::new(HashMap::new()),
        }
    }

    pub fn world(&self) -> &Arc<World> {
        &self.world
    }

    pub fn id(&self) -> Option<ModelId> {
        self.id.lock().unwrap().clone()
    }

    pub fn set_id(&self, id: ModelId) {
        *self.id.lock().unwrap() = Some(id);
    }

    pub fn set_parameter(&self, param: &str, value: &Value) {

        if let Some(param_descriptor) = self.descriptor.parameters().get(param) {

            if param_descriptor.datatype().is_compatible(&value) {

                self.parameters.lock().unwrap().insert(param.to_string(), value.clone());
            }
            else {
                panic!("Uncompatible value type for '{}'", param)
            }
        }
        else {
            panic!("Unknown parameter '{}'", param)
        }
    }

    pub fn get_parameter(&self, param: &str) -> Value {
        let borrowed_params = self.parameters.lock().unwrap();
        
        if let Some(value) = borrowed_params.get(param) {
            value.clone()
        }
        else {
            self.descriptor.parameters().get(param).unwrap().default().as_ref().unwrap().clone()
        }
    }
}

