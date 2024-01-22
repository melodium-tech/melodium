use super::DataTrait;
use crate::descriptor::Data as DataDescriptor;
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};
use std::sync::Arc;

pub trait Data: DataTrait + Debug + DowncastSync + Send + Sync {
    fn descriptor(&self) -> Arc<dyn DataDescriptor>;
}
impl_downcast!(sync Data);
