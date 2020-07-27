
use std::path::PathBuf;
use super::file::File;
use super::path::{Path, PathRoot};
use super::error::ScriptError;

pub struct Environment {
    pub main_path: PathBuf,
    pub standard_path: PathBuf,
    pub files: Vec<File>,
    pub errors: Vec<ScriptError>
}

impl Environment {
    pub fn new<P: Into<PathBuf>>(main_path: P, standard_path: P) -> Self {

        Self {
            main_path: main_path.into(),
            standard_path: standard_path.into(),
            files: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn build(&mut self) {

        // We create the "main" path.
        let mut main = Vec::new();
        main.push("main".to_string());
        main.push(self.main_path.file_stem().unwrap().to_str().unwrap().to_string());

        self.manage_file(Path::new(main), self.main_path.clone());

        while self.manage_inclusions() {}
    }

    fn manage_inclusions(&mut self) -> bool {

        let mut inclusions = Vec::new();
        for file in &self.files {

            for usage in &file.semantic.as_ref().unwrap().script.borrow().uses {

                let usage = usage.borrow();

                let canonical = self.get_canonical_path(&file.absolute_path, &usage.path);

                if let Some(canonical) = canonical {
                    let (canonical_path, canonical_pathbuf) = canonical;

                    let file_request = self.find_file(&canonical_pathbuf);
                    if file_request.is_none() {
                        inclusions.push((canonical_path, canonical_pathbuf));
                    }
                }
            }
        }

        for (canonical_path, canonical_pathbuf) in &inclusions {
            self.manage_file(canonical_path.clone(), canonical_pathbuf.clone());
        }

        !inclusions.is_empty()
    }

    fn manage_file(&mut self, path: Path, absolute_path: PathBuf) {

        // We check if the file is in the environment.
        let file_request = self.find_file(&absolute_path);

        // If it is not, then we include it.
        if file_request.is_none() {

            let mut file = File::new(path, absolute_path);

            file.read(); // TODO: Manage panic
            file.parse(); // TODO: Manage panic

            // We add it to the files list.
            self.files.push(file);
        }
    }

    fn find_file(&self, path: &PathBuf) -> Option<&File> {
        self.files.iter().find(|file| &file.absolute_path == path)
    }

    fn get_canonical_path(&self, includer_path: &PathBuf, path: &Path) -> Option<(Path, PathBuf)> {

        if path.is_valid() {
            if path.root() == PathRoot::Core {
                None
            }
            else if path.root() == PathRoot::Std {
                let mut canonical_path = self.standard_path.clone();

                //path.path().iter().map(|name| canonical_path.push(name));
                // Skipping "std" step, and pushing each intermediate name.
                for name in path.path().iter().skip(1) {
                    canonical_path.push(name);
                }

                canonical_path.set_extension("mel");

                Some((path.clone(), canonical_path))
            }
            else if path.root() == PathRoot::Main {
                let mut canonical_path = self.main_path.clone();

                // Removing filename.
                canonical_path.pop();
                // Skipping "main" step, and pushing each intermediate name.
                for name in path.path().iter().skip(1) {
                    canonical_path.push(name);
                }

                canonical_path.set_extension("mel");

                Some((path.clone(), canonical_path))
            }
            else if path.root() == PathRoot::Local {
                let mut path_from_main = Vec::new();
                path_from_main.push("main".to_string());
                let mut canonical_path = includer_path.clone();

                // Removing filename.
                canonical_path.pop();
                // Skipping "main" step, and pushing each intermediate name.
                for name in path.path().iter().skip(1) {
                    path_from_main.push(name.to_string());
                    canonical_path.push(name);
                }

                let path_from_main_obj = Path::new(path_from_main);
                canonical_path.set_extension("mel");

                Some((path_from_main_obj, canonical_path))
            }
            else {
                None
            }
        }
        else {
            None
        }

    }

    pub fn is_valid(&self) -> bool {
        !self.errors.is_empty()
    }
}
