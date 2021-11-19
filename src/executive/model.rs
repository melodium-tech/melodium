
use std::fmt::Debug;
use std::sync::Arc;
use std::collections::HashMap;
use downcast_rs::{DowncastSync, impl_downcast};
//use super::manager::Manager;
use super::value::Value;
use super::super::logic::descriptor::CoreModelDescriptor;

pub type ModelId = u64;

pub trait Model : Debug + DowncastSync + Send + Sync {

    fn descriptor(&self) -> &Arc<CoreModelDescriptor>;

    fn set_id(&self, id: ModelId);

    fn set_parameter(&self, param: &str, value: &Value);

    fn get_context_for(&self, source: &str) -> Vec<String>;

    fn initialize(&self);
    fn shutdown(&self);
}
impl_downcast!(sync Model);

