
use melodium_common::descriptor::Treatment;
use crate::error::LogicError;
use super::{BuildId, ContextualEnvironment, GenesisEnvironment, StaticBuildResult, DynamicBuildResult, CheckBuildResult, CheckEnvironment, CheckStep};
use std::sync::Arc;
use core::fmt::Debug;

pub trait Builder : Debug + Send + Sync {
    
    fn static_build(&self, host_treatment: Option<Arc<dyn Treatment>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError>;

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;
    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;

    fn check_dynamic_build(&self, build: BuildId, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult>;
    fn check_give_next(&self, within_build: BuildId, for_label: String, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult>;
}

/*
Notes: should builder be transformed to:
```
pub type BuildId = usize;

pub trait Builder : Debug + Send + Sync {
    
    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Option<StaticBuildResult>;

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;
    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;
}
```
*/
