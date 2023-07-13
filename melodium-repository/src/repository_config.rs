use crate::network::NetworkRepositoryConfiguration;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RepositoryConfig {
    pub repository_location: PathBuf,
    pub network: Option<NetworkRepositoryConfiguration>,
}
