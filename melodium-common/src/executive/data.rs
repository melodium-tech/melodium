use super::DataTrait;
use crate::descriptor::Data as DataDescriptor;
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};
use erased_serde::{serialize_trait_object, Serialize};
use std::sync::Arc;

pub trait Data: DataTrait + Serialize + Debug + DowncastSync + Send + Sync {
    fn descriptor(&self) -> Arc<dyn DataDescriptor>;
}
impl_downcast!(sync Data);
serialize_trait_object!(Data);
