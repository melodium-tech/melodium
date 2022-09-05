
//! Manages script location.
//! 

use std::path::PathBuf;

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum Base {
    FileSystem(PathBuf),
    Jeu(PathBuf),
    Internal(usize),
}

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

    pub fn get_all_mel_files(base: &Base) -> Vec<Self> {

        let mut locations = Vec::new();
        match base {
            Base::FileSystem(path) => {
                for entry in glob::glob(&format!("{}/**/*.mel", path.to_str().unwrap())).unwrap() {
                    match entry {
                        Ok(entry) => {
        
                            let absolute_path;
                            match entry.canonicalize() {
                                Ok(ap) => absolute_path = ap,
                                Err(_e) => {
                                    continue;
                                },
                            };
        
                            let relative_path = absolute_path.strip_prefix(&path).unwrap();

                            locations.push(Location::new(base.clone(), relative_path.to_owned()));
                        }
                        Err(_e) => {
                            continue;
                        }
                    }
                }
            },
            Base::Jeu(p) => {
                todo!()
            },
            Base::Internal(id) => {
                todo!()
            }
        }
        
        locations
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
