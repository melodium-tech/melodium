
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock};
use downcast_rs::{DowncastSync, impl_downcast};
use crate::logic::designer::ModelDesigner;
use super::identified::Identified;
use super::parameterized::Parameterized;
use super::buildable::Buildable;
use super::core_model::CoreModel;
use super::context::Context;

pub trait Model: Identified + Parameterized + Buildable + DowncastSync + Display + Debug + Send + Sync {
    fn is_core_model(&self) -> bool;
    fn core_model(&self) -> Arc<CoreModel>;
    fn sources(&self) -> &HashMap<String, Vec<Arc<Context>>>;
    fn designer(&self) -> Option<Arc<RwLock<ModelDesigner>>>;
}
impl_downcast!(sync Model);
