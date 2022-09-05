
//! Provides script instance management.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use super::location::{Base, Location};
use super::file::File;
use super::path::{Path, PathRoot};
use super::error::ScriptError;
use crate::logic::collection_pool::CollectionPool;
use crate::core::core_collection::core_collection;

/// Manage script instance.
/// 
/// Handle the whole instance of a Mélodium script, the files involved, and the logic associated with.
pub struct Instance {
    /// Location of the main script file.
    pub main: Location,
    /// Base for the standard library.
    pub standard: Base,
    /// Files used in the instance.
    /// 
    /// This include the main file, and may be empty for many reasons (instance not built, or errors, etc.)
    pub files: Vec<File>,
    /// Errors present in the instance.
    pub errors: HashMap<Location, ScriptError>,

    pub logic_collection: Option<Arc<CollectionPool>>,
}

impl Instance {
    /// Create a new instance, based on main file and standard library path given.
    /// 
    /// This does not build anything, nor check paths, just create an empty instance.
    /// 
    /// * `main`: location to the main script file.
    /// * `standard`: base for the standard library root.
    pub fn new(main: Location, standard: Base) -> Self {

        Self {
            main,
            standard,
            files: Vec::new(),
            errors: HashMap::new(),
            logic_collection: None,
        }
    }

    /// Build the instance by main script.
    /// 
    /// The instance build and parse from the main file and all the `use`s.
    /// It can be used to create an instance aimed to be executed, only the required files are parsed.
    /// After building instance, check if it is valid and what errors occured.
    pub fn build_by_main(&mut self) {

        // We create the "main" path.
        let mut main = Vec::new();
        main.push("main".to_string());
        main.push(self.main.path.file_stem().unwrap().to_str().unwrap().to_string());

        self.manage_file(self.main.clone(), Path::new(main));

        self.build();
    }

    /// Build the instance from the whole stdlib.
    /// 
    /// The instance build and parse all files found in the stdlib. It ignores the main path.
    /// After building instance, check if it is valid and what errors occured.
    pub fn build_all_std(&mut self) {

        self.build_all_prefix("std", &self.standard.clone());

        self.build();
    }

    /// Build the instance from the whole main.
    /// 
    /// The instance build and parse all files found in the main path.
    /// After building instance, check if it is valid and what errors occured.
    pub fn build_all_main(&mut self) {

        self.build_all_prefix("main", &self.main.base.clone());

        self.build();
    }

    /// Build the instance from the whole stdlib and main.
    /// 
    /// The instance build and parse all files found in the stdlib and main path.
    /// After building instance, check if it is valid and what errors occured.
    pub fn build_all(&mut self) {

        self.build_all_prefix("std", &self.standard.clone());
        self.build_all_prefix("main", &self.main.base.clone());

        self.build();
    }

    pub fn collection(&self) -> &Option<Arc<CollectionPool>> {
        &self.logic_collection
    }

    pub fn errors(&self) -> &HashMap<Location, ScriptError> {
        &self.errors
    }

    fn build_all_prefix(&mut self, prefix: &str, base: &Base) {

        for location in Location::get_all_mel_files(base) {

            let mut path_steps: Vec<&str> = location.path.to_str().unwrap().strip_suffix(".mel").unwrap().split('/').collect();
            path_steps.insert(0, prefix);
            let path = Path::new(path_steps.iter().map(|s| s.to_string()).collect());

            self.manage_file(location, path);
        }
    }

    fn build(&mut self) {

        while self.manage_inclusions() {
            if !self.errors.is_empty() {
                return;
            }
        }

        self.make_descriptors();

        self.make_designs();
    }

    /// Manage inclusions of files in the instance.
    /// 
    /// This method checks what files/entities are used in _already included_ files, and check if they are present in the instance,
    /// if not, it includes the file relatively from the `use` instruction, but *do not* manage inclusions of the newly included files.
    /// 
    /// Returns `true` if new files were included by the call, or `false` if not.
    /// 
    /// The aim of this method is to be called repeatedly until it return `false`, and optionally manage iterations to avoid infinite loop (and make debug easier).
    fn manage_inclusions(&mut self) -> bool {

        let mut inclusions = Vec::new();
        for file in &self.files {

            for usage in &file.semantic.as_ref().unwrap().script.read().unwrap().uses {

                let usage = usage.read().unwrap();

                let canonical = self.get_canonical_path(&file.location, &usage.path);

                if let Some(canonical) = canonical {
                    let (location, path) = canonical;

                    let file_request = self.find_file(&location);
                    if file_request.is_none() {
                        inclusions.push((location, path));
                    }
                }
            }
        }

        for (location, path) in &inclusions {
            self.manage_file(location.clone(), path.clone());
        }

        !inclusions.is_empty()
    }

    /// Manage inclusion of a file in the instance.
    /// 
    /// If the file is not already present, it includes it.
    /// It makes the new file being read and parsed _before_ pushing it in instance.
    /// 
    /// * `path`: canonical path of the file inside the Mélodium instance (see [File::path](super::file::File::path)).
    /// * `absolute_path`: absolute system path to the file in filesystem (see [File::absolute_path](super::file::File::absolute_path)).
    fn manage_file(&mut self, location: Location, path: Path) {

        // We check if the file is in the instance.
        let file_request = self.find_file(&location);

        // If it is not, then we include it.
        if file_request.is_none() {

            let mut file = File::new(location.clone(), path);

            let reading_result = file.read();

            if reading_result.is_err() {
                self.errors.insert(location.clone(), ScriptError::file(reading_result.unwrap_err().to_string()));
                return;
            }

            let parsing_result = file.parse();

            if parsing_result.is_err() {
                self.errors.insert(location.clone(), parsing_result.unwrap_err());
                return;
            }

            // We add it to the files list.
            self.files.push(file);
        }
    }

    /// Tells if a file exists in the instance, and give reference to it.
    fn find_file(&self, location: &Location) -> Option<&File> {
        self.files.iter().find(|file| {
            &file.location == location
        })
    }

    /// Get the location and path of a file based on includer path  and location.
    /// 
    /// Returns a tuple of (location and canonical path), or none if nothing could be determined.
    fn get_canonical_path(&self, includer_location: &Location, path: &Path) -> Option<(Location, Path)> {

        if path.is_valid() {
            if path.root() == PathRoot::Core {
                None
            }
            else if path.root() == PathRoot::Std {
                let mut location = Location::new(self.standard.clone(), PathBuf::new());

                //path.path().iter().map(|name| canonical_path.push(name));
                // Skipping "std" step, and pushing each intermediate name.
                for name in path.path().iter() {
                    location.path.push(name);
                }

                location.path.set_extension("mel");

                Some((location, path.clone()))
            }
            else if path.root() == PathRoot::Main {
                let mut location = Location::new(self.main.base.clone(), PathBuf::new());

                // Removing filename.
                //canonical_path.pop();
                // Skipping "main" step, and pushing each intermediate name.
                for name in path.path().iter().skip(1) {
                    location.path.push(name);
                }

                location.path.set_extension("mel");

                Some((location, path.clone()))
            }
            else if path.root() == PathRoot::Local {
                let mut path_from_main = Vec::new();
                path_from_main.push("main".to_string());
                let mut canonical_path = includer_location.path.clone();

                // Removing filename.
                canonical_path.pop();
                // Skipping "main" step, and pushing each intermediate name.
                for name in path.path().iter().skip(1) {
                    path_from_main.push(name.to_string());
                    canonical_path.push(name);
                }

                let path_from_main_obj = Path::new(path_from_main);
                canonical_path.set_extension("mel");

                Some((Location::new(includer_location.base.clone(), canonical_path), path_from_main_obj))
            }
            else {
                None
            }
        }
        else {
            None
        }

    }

    fn make_descriptors(&mut self) {

        let mut logic_collection = core_collection().clone();

        // We declare first all models
        for file in &self.files {
            let borrowed_script = &file.semantic.as_ref().unwrap().script.read().unwrap();

            for (_, rc_model) in &borrowed_script.models {

                let borrowed_model = rc_model.read().unwrap();

                if let Err(e) = borrowed_model.make_descriptor(&mut logic_collection) {
                    self.errors.insert(file.location.clone(), e);
                }
            }
        }

        // Then we declare all sequences
        for file in &self.files {
            let borrowed_script = &file.semantic.as_ref().unwrap().script.read().unwrap();

            for (_, rc_sequence) in &borrowed_script.sequences {

                let borrowed_sequence = rc_sequence.read().unwrap();

                if let Err(e) = borrowed_sequence.make_descriptor(&mut logic_collection) {
                    self.errors.insert(file.location.clone(), e);
                }
            }
        }

        self.logic_collection = Some(Arc::new(logic_collection));
    }

    fn make_designs(&mut self) {

        // No order is required there, should be parrelizable easily

        for file in &self.files {
            let borrowed_script = &file.semantic.as_ref().unwrap().script.read().unwrap();

            for (_, rc_model) in &borrowed_script.models {

                let borrowed_model = rc_model.read().unwrap();

                if let Err(e) = borrowed_model.make_design(self.logic_collection.as_ref().unwrap()) {
                    self.errors.insert(file.location.clone(), e);
                }
            }

            for (_, rc_sequence) in &borrowed_script.sequences {

                let borrowed_sequence = rc_sequence.read().unwrap();

                if let Err(e) = borrowed_sequence.make_design(&self.logic_collection.as_ref().unwrap()) {
                    self.errors.insert(file.location.clone(), e);
                }
            }
        }
    }

    /// Tell if environment is valid.
    pub fn is_valid(&self) -> bool {
        !self.errors.is_empty()
    }
}
