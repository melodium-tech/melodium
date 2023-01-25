
use crate::Loader;
use crate::package::package::Package;
use crate::content::{Content, ContentError};
use melodium_common::descriptor::{Collection, Identifier, LoadingError};
use semver::Version;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use std::fs::{metadata, read, read_to_string};
use toml::{Table, Value};
use glob::{glob_with, MatchOptions};

pub struct FsPackage {
    path: PathBuf,
    name: String,
    version: Version,
    contents: RwLock<HashMap<PathBuf, Content>>,
    errors: RwLock<Vec<ContentError>>,
}

impl FsPackage {
    pub fn new(path: &Path) -> Result<Self, LoadingError> {

        let location = metadata(path).map_err(|_| LoadingError::NoPackage)?;

        if location.is_dir() {

            let mut composition_path = path.to_path_buf();
            composition_path.push("Compo.toml");

            let composition = read_to_string(&composition_path).map_err(|_| LoadingError::NoPackage)?;
            let composition = composition.parse::<Table>().map_err(|_| LoadingError::NoPackage)?;

            if let (Value::String(name), Ok(version)) = (composition.get("name").ok_or(LoadingError::NoPackage)?, Version::parse(composition.get("version").ok_or(LoadingError::NoPackage)?.as_str().ok_or(LoadingError::NoPackage)?)) {
                Ok(Self { path: path.to_path_buf(), name: name.clone(), version, contents: RwLock::new(HashMap::new()), errors: RwLock::new(Vec::new()) })
            } else {
                Err(LoadingError::NoPackage)
            }
        } else {
            Err(LoadingError::NoPackage)
        }
        
    }

    fn insure_content(&self, designation: &Path) -> Result<(), LoadingError> {

        if self.contents.read().unwrap().contains_key(designation) {
            return Ok(())
        }

        let mut full_path = self.path.clone();
        full_path.push(designation);
        let raw = read(full_path).map_err(|_| LoadingError::NotFound)?;

        let result_content = Content::new(&designation.as_os_str().to_string_lossy(), &raw);

        match result_content {
            Ok(content) => {
                self.contents.write().unwrap().insert(designation.to_path_buf(), content);
                Ok(())
            },
            Err(error) => {
                self.errors.write().unwrap().push(error);
                Err(LoadingError::NotFound)
            }
        }

    }

    fn all_contents(&self) -> Result<(), LoadingError> {

        let pattern = format!("{}/**.mel", self.path.to_string_lossy().to_string());

        let options = MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: true,
        };

        for entry in glob_with(&pattern, options).map_err(|_| LoadingError::NotFound)? {
            match entry {
                Ok(path) => self.insure_content(path.strip_prefix(&self.path).map_err(|_| LoadingError::NotFound)?)?,
                Err(_) => {},
            }
        }

        Ok(())
    }

    fn insure_loading(loader: &Loader, identifiers: Vec<Identifier>) -> Result<(), LoadingError> {

        for identifier in identifiers {
            loader.get_with_load(&identifier)?;
        }

        Ok(())
    }

    fn designation(identifier: &Identifier) -> PathBuf {
        PathBuf::from(format!("{}.mel", identifier.path().join("/")))
    }
}

impl Package for FsPackage {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &Version {
        &self.version
    }

    fn embedded_collection(&self, _loader: &Loader) -> Result<Collection, LoadingError> {
        Ok(Collection::new())
    }

    fn full_collection(&self, loader: &Loader) -> Result<Collection, LoadingError> {
        
        let identifiers = self.all_identifiers(loader)?;

        let mut collection = Collection::new();
        let mut error = false;
        for identifier in &identifiers {
            if collection.get(identifier).is_none() {
                match self.element(loader, identifier) {
                    Ok(specific_collection) => {
                        for identifier in &specific_collection.identifiers() {
                            collection.insert(specific_collection.get(identifier).unwrap().clone());
                        }
                    },
                    Err(_) => {
                        error = true;
                    }
                }
            }
        }

        
        if error {
            Err(LoadingError::ContentError)
        } else {
            Ok(collection)
        }
    }

    fn all_identifiers(&self, _loader: &Loader) -> Result<Vec<Identifier>, LoadingError> {
        self.all_contents()?;

        let mut identifiers = Vec::new();
        self.contents.read().unwrap().iter().for_each(|(_, content)| identifiers.extend(content.provide()));

        Ok(identifiers)
    }

    fn element(&self, loader: &Loader, identifier: &Identifier) -> Result<Collection, LoadingError> {
        
        let designation = Self::designation(identifier);
        self.insure_content(&designation)?;

        if let Some(content) = self.contents.read().unwrap().get(&designation) {
            if let Ok(_guard) = content.try_lock() {
                let needs = content.require();
                Self::insure_loading(loader, needs)?;

                let mut collection = loader.collection().clone();
                match content.insert_descriptors(&mut collection) {
                    Ok(()) => Ok(collection),
                    Err(error) => {
                        self.errors.write().unwrap().push(error);
                        Err(LoadingError::ContentError)
                    }
                }
            }
            else {
                Err(LoadingError::CircularReference)
            }
        }
        else {
            Err(LoadingError::NotFound)
        }
    }

    fn make_building(&self, collection: &Arc<Collection>) -> Result<(), LoadingError> {
        
        let contents = self.contents.read().unwrap();
        let mut content_error = false;
        for (_, content) in contents.iter() {
            if let Err(error) = content.make_design(collection) {
                self.errors.write().unwrap().push(error);
                content_error = true;
            }
        }

        if content_error {
            Err(LoadingError::ContentError)
        } else {
            Ok(())
        }
    }

    fn errors(&self) -> Vec<ContentError> {
        self.errors.read().unwrap().clone()
    }
}
