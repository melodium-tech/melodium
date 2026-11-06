use crate::building::BuildId;
use crate::error::LogicErrors;
use core::fmt::{Debug, Display, Formatter, Result};
use melodium_common::descriptor::Identifier;
use std::collections::HashSet;
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

/// State of a walk checking the tracks of a source, shared by all its steps, so that each
/// build is checked once, whatever the number of paths leading to it.
#[derive(Debug, Default)]
pub struct CheckState {
    /// Steps of the path being checked, a step met again on it being a loop.
    pub path: Vec<CheckStep>,
    /// Steps whose whole following is checked.
    pub checked: HashSet<CheckStep>,
    /// Followings of labels within builds that are checked, as treatment, build and label.
    pub followed: HashSet<(Identifier, BuildId, String)>,
}
