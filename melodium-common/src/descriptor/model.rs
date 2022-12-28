
use std::sync::Arc;
use std::collections::HashMap;
use core::fmt::{Debug, Display};
use downcast_rs::{DowncastSync, impl_downcast};
use super::{Buildable, Context, Documented, Identified, Parameterized, ModelBuildMode};

pub trait Model : Identified + Documented + Parameterized + Buildable<ModelBuildMode> + DowncastSync + Display + Debug + Send + Sync {
    fn is_core_model(&self) -> bool;
    fn base_model(&self) -> Arc<dyn Model>;
    fn sources(&self) -> &HashMap<String, Vec<Arc<Context>>>;
    fn as_identified(&self) -> Arc<dyn Identified>;
    fn as_parameterized(&self) -> Arc<dyn Parameterized>;
}
impl_downcast!(sync Model);
