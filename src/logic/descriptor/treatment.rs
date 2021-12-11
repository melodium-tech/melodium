
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;
use downcast_rs::{DowncastSync, impl_downcast};
use super::identified::Identified;
use super::parameterized::Parameterized;
use super::buildable::Buildable;
use super::input::Input;
use super::output::Output;
use super::core_model::CoreModel;
use super::requirement::Requirement;

pub trait Treatment: Identified + Parameterized + Buildable + DowncastSync + Debug + Send + Sync {
    fn inputs(&self) -> &HashMap<String, Input>;
    fn outputs(&self) -> &HashMap<String, Output>;
    fn models(&self) -> &HashMap<String, Arc<CoreModel>>;
    fn requirements(&self) -> &HashMap<String, Requirement>;
    fn source_from(&self) -> &HashMap<Arc<CoreModel>, Vec<String>>;
    fn as_buildable(&self) -> Arc<dyn Buildable>;
}
impl_downcast!(sync Treatment);
