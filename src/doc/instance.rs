
use std::collections::HashMap;
use std::path::PathBuf;
use glob::glob;
use crate::script::file::File;
use crate::script::path::{Path, PathRoot};
use crate::script::error::ScriptError;

pub struct Instance {
    pub root: PathRoot,
    pub entry_path: PathBuf,
    pub output_path: PathBuf,
    /// Files used in the instance.
    pub script_files: Vec<File>,
}

impl Instance {

    pub fn new(root: PathRoot, entry_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            root,
            entry_path,
            output_path,
            script_files: Vec::new(),
        }
    }

    pub fn parse_files(&mut self) -> Result<(), (Vec<std::io::Error>, HashMap<PathBuf, ScriptError>)> {

        let mut io_errors = Vec::new();
        let mut script_errors = HashMap::new();

        for entry in glob(&format!("{}/**/*.mel", self.entry_path.to_str().unwrap())).unwrap() {
            match entry {
                Ok(entry) => {

                    let absolute_path;
                    match entry.canonicalize() {
                        Ok(ap) => absolute_path = ap,
                        Err(e) => {
                            io_errors.push(e);
                            continue;
                        },
                    };
                    let relative_path = absolute_path.strip_prefix(&self.entry_path).unwrap();
                    let mut path_steps: Vec<&str> = relative_path.to_str().unwrap().strip_suffix(".mel").unwrap().split('/').collect();
                    path_steps.insert(0, match self.root {
                        PathRoot::Main => "main",
                        PathRoot::Std => "std",
                        _ => "",
                    });
                    let path = Path::new(path_steps.iter().map(|s| s.to_string()).collect());


                    let mut file = File::new(path, absolute_path);

                    if let Err(e) = file.read() {
                        io_errors.push(e);
                        continue;
                    }

                    if let Err(e) = file.parse() {
                        script_errors.insert(file.absolute_path, e);
                        continue;
                    }

                    self.script_files.push(file);
                },
                Err(e) => {
                    io_errors.push(e.into_error());
                },
            }
        }

        if io_errors.is_empty() && script_errors.is_empty() {
            Ok(())
        }
        else {
            Err((io_errors, script_errors))
        }
    }

    pub fn output_doc(&self) -> std::io::Result<()> {

        for script in &self.script_files {

            let output_path = self.get_output_path(&script.path);
            let mut file = std::fs::File::create(output_path)?;
        }

        Ok(())
    }

    fn get_output_path(&self, path: &Path) -> PathBuf {

        let mut os_path = self.output_path.clone();

        for step in path.path() {
            os_path = os_path.join(step);
        }

        os_path.join(".md")
    }

    fn model_doc() -> String {

        "".to_string()
    }

    fn sequence_doc() -> String {

        "".to_string()
    }
}