
pub mod contextual_environment;
pub mod genesis_environment;
pub mod result;
pub mod check;

pub type BuildId = usize;
pub use contextual_environment::ContextualEnvironment;
pub use genesis_environment::GenesisEnvironment;
pub use result::{StaticBuildResult, DynamicBuildResult};
pub use check::{CheckBuild, CheckBuildResult, CheckEnvironment, CheckStep};

// To move where appliable
use std::collections::HashMap;
pub type FeedingInputs = HashMap<String, Vec<Input>>;
