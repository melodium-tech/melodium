
use core::fmt::Debug;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use melodium_common::descriptor::Identifier;
use crate::building::BuildId;
use crate::error::LogicError;

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
    pub identifier: Identifier,
    pub build_id: BuildId,
}

#[derive(Debug, Clone)]
pub struct CheckEnvironment {
    pub contextes: Vec<String>
}
