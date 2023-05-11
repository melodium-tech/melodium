use core::fmt::Debug;
use melodium_common::descriptor::{Collection, Identifier, Parameterized};
use std::sync::Arc;

pub trait Scope: Send + Sync + Debug {
    fn descriptor(&self) -> Arc<dyn Parameterized>;
    fn identifier(&self) -> Identifier;
    fn collection(&self) -> Arc<Collection>;
}
