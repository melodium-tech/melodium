// TODO Those checks are now useless, need some cleanup.
#![allow(unused)]

use crate::building::BuildId;
use crate::error::LogicErrors;
use core::fmt::{Debug, Display, Formatter, Result};
use melodium_common::descriptor::Identifier;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct CheckBuild {
    pub host_id: Option<Identifier>,
    pub label: String,
}
impl CheckBuild {
    pub fn new(host_id: Option<Identifier>, label: &str) -> Self {
        Self {
            host_id,
            label: label.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckBuildResult {
    pub checked_builds: Vec<Arc<RwLock<CheckBuild>>>,
    pub build: Arc<RwLock<CheckBuild>>,
    pub errors: LogicErrors,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckStep {
    pub identifier: Identifier,
    pub build_id: BuildId,
}

impl Display for CheckStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[{}; {}]", self.build_id, self.identifier)
    }
}

#[derive(Debug, Clone)]
pub struct CheckEnvironment {
    pub contextes: Vec<String>,
}
