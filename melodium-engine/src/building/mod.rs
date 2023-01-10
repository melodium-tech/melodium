pub mod builder;
pub mod check;
pub mod contextual_environment;
pub mod genesis_environment;
pub mod model;
pub mod result;
pub mod treatment;

pub type BuildId = usize;
pub use builder::Builder;
pub use check::{CheckBuild, CheckBuildResult, CheckEnvironment, CheckStep};
pub use contextual_environment::ContextualEnvironment;
pub use genesis_environment::GenesisEnvironment;
pub use model::{CompiledBuilder as CompiledModelBuilder, DesignedBuilder as DesignedModelBuilder};
pub use result::{DynamicBuildResult, StaticBuildResult};
pub use treatment::{
    CompiledBuilder as CompiledTreatmentBuilder, DesignedBuilder as DesignedTreatmentBuilder,
    SourceBuilder as SourceTreatmentBuilder,
};

// To move where appliable
use crate::transmission::Input;
use std::collections::HashMap;
pub type FeedingInputs = HashMap<String, Vec<Input>>;
