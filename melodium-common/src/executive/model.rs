
use core::fmt::Debug;
use std::sync::Arc;
use downcast_rs::{DowncastSync, impl_downcast};
use crate::descriptor::Model as ModelDescriptor;
use crate::executive::Value;

pub type ModelId = u64;

pub trait Model : Debug + DowncastSync + Send + Sync {

    fn descriptor(&self) -> Arc<dyn ModelDescriptor>;

    fn id(&self) -> Option<ModelId>;
    fn set_id(&self, id: ModelId);

    fn set_parameter(&self, param: &str, value: &Value);

    fn initialize(&self);
    fn shutdown(&self);
}
impl_downcast!(sync Model);
