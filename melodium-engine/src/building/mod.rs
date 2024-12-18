pub(crate) mod builder;
pub(crate) mod check;
pub(crate) mod contextual_environment;
pub(crate) mod genesis_environment;
pub(crate) mod host;
pub(crate) mod model;
pub(crate) mod result;
pub(crate) mod treatment;

pub type BuildId = usize;
pub use builder::Builder;
pub use check::{CheckBuild, CheckBuildResult, CheckEnvironment, CheckStep};
pub use contextual_environment::ContextualEnvironment;
pub use genesis_environment::GenesisEnvironment;
pub use host::HostTreatment;
pub use result::{DynamicBuildResult, StaticBuildResult};

// To move where appliable
use crate::transmission::Input;
use std::collections::HashMap;
pub type FeedingInputs = HashMap<String, Vec<Input>>;
