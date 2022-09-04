
//! Manages script location.
//! 

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Base {
    FileSystem(PathBuf),
    Jeu(PathBuf),
    Internal(usize),
}

#[derive(Clone, Debug)]
pub struct Location {
    pub base: Base,
    pub path: PathBuf,
}

impl Location {

    pub fn new(base: Base) -> Self {
        Self {
            base,
            path: PathBuf::new(),
        }
    }

    pub fn read_to_string(&self) -> std::io::Result<String> {
        match self.base {
            Base::FileSystem(p) => {

                let mut complete_path = p.clone();
                complete_path.push(self.path);

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
