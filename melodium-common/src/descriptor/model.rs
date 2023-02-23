use super::{Buildable, Context, Documented, Identified, ModelBuildMode, Parameterized};
use core::fmt::{Debug, Display};
use downcast_rs::{impl_downcast, DowncastSync};
use std::collections::HashMap;
use std::sync::Arc;

pub trait Model:
    Identified
    + Documented
    + Parameterized
    + Buildable<ModelBuildMode>
    + DowncastSync
    + Display
    + Debug
    + Send
    + Sync
{
    fn is_core_model(&self) -> bool;
    fn base_model(&self) -> Option<Arc<dyn Model>>;
    fn sources(&self) -> &HashMap<String, Vec<Arc<dyn Context>>>;
    fn as_identified(&self) -> Arc<dyn Identified>;
    fn as_buildable(&self) -> Arc<dyn Buildable<ModelBuildMode>>;
    fn as_parameterized(&self) -> Arc<dyn Parameterized>;
}
impl_downcast!(sync Model);
