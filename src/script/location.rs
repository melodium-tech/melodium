
//! Manages script location.
//! 

use std::path::PathBuf;
use super::base::Base;

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct Location {
    pub base: Base,
    pub path: PathBuf,
}

impl Location {

    pub fn new(base: Base, path: PathBuf) -> Self {
        Self {
            base,
            path,
        }
    }

    pub fn read_to_string(&self) -> std::io::Result<String> {
        self.base.read_to_string(&self.path)
    }
}
