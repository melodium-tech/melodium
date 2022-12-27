
use core::fmt::Debug;
use std::sync::Arc;
use downcast_rs::{DowncastSync, impl_downcast};
use super::Collection;

pub trait Designer : Debug + DowncastSync + Send + Sync {
    fn set_collection(&self, collection: Arc<Collection>);
}
impl_downcast!(sync Designer);
