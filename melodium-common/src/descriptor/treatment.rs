
use core::fmt::{Debug, Display};
use std::collections::HashMap;
use std::sync::Arc;
use downcast_rs::{DowncastSync, impl_downcast};
use super::{Buildable, Context, Documented, Identified, Input, Model, Output, Parameterized, TreatmentBuildMode};

pub trait Treatment: Identified + Documented + Parameterized + Buildable<TreatmentBuildMode> + DowncastSync + Display + Debug + Send + Sync {
    fn inputs(&self) -> &HashMap<String, Input>;
    fn outputs(&self) -> &HashMap<String, Output>;
    fn models(&self) -> &HashMap<String, Arc<dyn Model>>;
    fn contexts(&self) -> &HashMap<String, Context>;
    fn source_from(&self) -> &HashMap<Arc<dyn Model>, Vec<String>>;
    fn as_identified(&self) -> Arc<dyn Identified>;
    fn as_buildable(&self) -> Arc<dyn Buildable<TreatmentBuildMode>>;
    fn as_parameterized(&self) -> Arc<dyn Parameterized>;
}
impl_downcast!(sync Treatment);
