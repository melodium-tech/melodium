
use std::fmt::Debug;
use std::sync::Arc;
use downcast_rs::{DowncastSync, impl_downcast};
use super::identified::Identified;
use super::parameterized::Parameterized;
use super::buildable::Buildable;
use super::core_model::CoreModel;

pub trait Model: Identified + Parameterized + Buildable + DowncastSync + Debug + Send + Sync {
    fn is_core_model(&self) -> bool;
    fn core_model(&self) -> Arc<CoreModel>;
}
impl_downcast!(sync Model);
