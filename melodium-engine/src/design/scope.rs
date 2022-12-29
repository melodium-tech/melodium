
use core::fmt::Debug;
use std::sync::Arc;
use melodium_common::descriptor::{Collection, Parameterized};

pub trait Scope : Send + Sync + Debug {

    fn descriptor(&self) -> Arc<dyn Parameterized>;
    fn collections(&self) -> Arc<Collection>;
}
