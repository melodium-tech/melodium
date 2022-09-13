
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
        match &self.base {
            Base::FileSystem(p) => {

                let mut complete_path = p.clone();
                complete_path.push(self.path.clone());

                std::fs::read_to_string(complete_path.canonicalize()?)
            },
            Base::Jeu(p) => {
                todo!()
            },
            Base::Internal(id) => {
                todo!()
            }
        }
    }
}
