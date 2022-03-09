

use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use std::fmt::Debug;
use super::super::error::LogicError;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::value::Value;
use super::super::super::executive::model::Model;
use super::super::super::executive::transmitter::Transmitter;
use super::super::super::executive::input::Input;
use super::super::super::executive::future::*;
use super::super::descriptor::TreatmentDescriptor;
use super::super::descriptor::IdentifierDescriptor;

pub type BuildId = u64;
pub type FeedingInputs = HashMap<String, Vec<Input>>;

#[derive(Debug)]
pub enum StaticBuildResult {
    Model(Arc<dyn Model>),
    Build(BuildId),
}

pub struct DynamicBuildResult {
    pub prepared_futures: Vec<TrackFuture>,
    pub feeding_inputs: FeedingInputs,
}

impl DynamicBuildResult {

    pub fn new() -> Self {
        Self {
            prepared_futures: Vec::new(),
            feeding_inputs: HashMap::new(),
        }
    }
}

impl Debug for DynamicBuildResult {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicBuildResult")
         .field("feeding_inputs", &self.feeding_inputs)
         .field("prepared_futures", &self.prepared_futures.len())
         .finish()
    }
}

#[derive(Clone, Debug)]
pub struct CheckBuild {
    pub fed_inputs: HashMap<String, bool>,
}
impl CheckBuild {
    pub fn new() -> Self {
        Self {
            fed_inputs: HashMap::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckBuildResult {
    pub checked_builds: Vec<Arc<RwLock<CheckBuild>>>,
    pub build: Arc<RwLock<CheckBuild>>,
    pub errors: Vec<LogicError>,
}

impl CheckBuildResult {
    pub fn new() -> Self {
        Self {
            checked_builds: Vec::new(),
            build: Arc::new(RwLock::new(CheckBuild::new())),
            errors: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckStep {
    pub identifier: IdentifierDescriptor,
    pub build_id: BuildId,
}

#[derive(Debug, Clone)]
pub struct CheckEnvironment {
    pub contextes: Vec<String>
}

pub trait Builder : Debug + Send + Sync {
    
    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError>;

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;
    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;

    fn check_dynamic_build(&self, build: BuildId, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult>;
    fn check_give_next(&self, within_build: BuildId, for_label: String, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult>;
}
