
use std::path::{Path, PathBuf};
use std::collections::{HashMap, hash_map::Entry};
use std::sync::{Arc, Mutex};
use std::io::Result;
use std::fs::File;
use crate::jeu::Jeu;
use super::location::Location;

lazy_static! {
    static ref JEUX: Mutex<HashMap<PathBuf, Arc<Jeu>>> = Mutex::new(HashMap::new());
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum Base {
    FileSystem(PathBuf),
    Jeu(PathBuf),
    Internal(usize),
}

impl Base {
    pub fn get_all_mel_files(&self) -> Result<Vec<Location>> {

        let mut locations = Vec::new();
        match self {
            Base::FileSystem(path) => {
                for entry in glob::glob(&format!("{}/**/*.mel", path.to_str().unwrap())).unwrap() {
                    match entry {
                        Ok(entry) => {
        
                            let absolute_path;
                            match entry.canonicalize() {
                                Ok(ap) => absolute_path = ap,
                                Err(e) => {
                                    return Err(e)
                                },
                            };
        
                            let relative_path = absolute_path.strip_prefix(&path).unwrap();

                            locations.push(Location::new(self.clone(), relative_path.to_owned()));
                        }
                        Err(e) => {
                            return Err(e.into_error())
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
        
        Ok(locations)
    }

    fn get_jeu(path: &Path) -> Result<Arc<Jeu>> {

        match JEUX.lock().unwrap().entry(path.to_path_buf()) {
            Entry::Occupied(entry) => Ok(Arc::clone(entry.get())),
            Entry::Vacant(entry) => {

                let jeu = Arc::new(Jeu::new(File::open(path)?)?);
                Ok(Arc::clone(entry.insert(jeu)))
            }
        }
    }
}
