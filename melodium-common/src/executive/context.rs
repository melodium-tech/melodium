use crate::descriptor::Context as ContextDescriptor;
use crate::executive::Value;
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};
use std::sync::Arc;

pub trait Context: Debug + DowncastSync + Send + Sync {
    fn descriptor(&self) -> Arc<ContextDescriptor>;
    fn set_value(&mut self, name: &str, value: Value);
    fn get_value(&self, name: &str) -> &Value;
}
impl_downcast!(sync Context);
