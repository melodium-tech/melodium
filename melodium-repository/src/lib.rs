#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub mod error;
pub mod global;
pub mod network;
pub mod repository;
pub mod repository_config;
pub mod technical;
pub use error::{RepositoryError, RepositoryResult};
pub use repository::Repository;
pub use repository_config::RepositoryConfig;
#[cfg_attr(docsrs, doc(cfg(feature = "cargo")))]
#[cfg(feature = "cargo")]
pub mod utils;
