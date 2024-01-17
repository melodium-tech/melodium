use crate::descriptor::Object as ObjectDescriptor;
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};
use std::sync::Arc;

pub trait Object: Debug + DowncastSync + Send + Sync {
    fn descriptor(&self) -> Arc<dyn ObjectDescriptor>;
}
impl_downcast!(sync Object);
