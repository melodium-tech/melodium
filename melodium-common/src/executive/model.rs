use crate::descriptor::Model as ModelDescriptor;
use crate::executive::Value;
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};
use std::{collections::HashMap, sync::Arc};

pub type ModelId = usize;

pub trait Model: Debug + DowncastSync + Send + Sync {
    fn descriptor(&self) -> Arc<dyn ModelDescriptor>;

    fn id(&self) -> Option<ModelId>;
    fn set_id(&self, id: ModelId);

    fn set_parameter(&self, param: &str, value: Value);

    fn initialize(&self);
    fn shutdown(&self);

    fn invoke_source(&self, source: &str, params: HashMap<String, Value>);
}
impl_downcast!(sync Model);
