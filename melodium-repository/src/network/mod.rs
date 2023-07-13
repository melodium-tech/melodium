mod config;
#[cfg(feature = "network")]
pub(crate) mod remote;

pub use config::Config as NetworkRepositoryConfiguration;
