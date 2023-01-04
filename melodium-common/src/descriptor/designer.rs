use super::Collection;
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};
use std::sync::Arc;

pub trait Designer: Debug + DowncastSync + Send + Sync {
    fn set_collection(&mut self, collection: Arc<Collection>);
}
impl_downcast!(sync Designer);
