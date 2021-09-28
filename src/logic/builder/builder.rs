

use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use std::fmt::Debug;
use super::super::error::LogicError;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::value::Value;
use super::super::super::executive::model::Model;
use super::super::super::executive::transmitter::Transmitter;
use super::super::super::executive::future::Future;
use super::super::descriptor::TreatmentDescriptor;

pub type BuildId = u64;

#[derive(Debug)]
pub enum StaticBuildResult {
    Model(Arc<dyn Model>),
    Build(BuildId),
}

pub struct DynamicBuildResult {
    pub prepared_futures: Vec<Box<Future>>,
    pub feeding_inputs: HashMap<String, Transmitter>,
}

impl Debug for DynamicBuildResult {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicBuildResult")
         .field("feeding_inputs", &self.feeding_inputs)
         .field("prepared_futures", &self.prepared_futures.len())
         .finish()
    }
}

#[derive(Debug)]
pub struct EnvironmentSample {
    pub contextes: Vec<String>
}

pub trait Builder : Debug + Send + Sync {
    
    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError>;

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;
    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;

    fn check_dynamic_build(&self, build: BuildId, ) -> Vec<LogicError>;
    fn check_give_next(&self, within_build: BuildId, for_label: String, ) -> Vec<LogicError>;
}
