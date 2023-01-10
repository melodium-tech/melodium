use core::fmt::Debug;
use melodium_common::descriptor::{Collection, Parameterized};
use std::sync::Arc;

pub trait Scope: Send + Sync + Debug {
    fn descriptor(&self) -> Arc<dyn Parameterized>;
    fn collection(&self) -> Option<Arc<Collection>>;
}
