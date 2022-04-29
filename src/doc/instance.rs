
use std::io::Write;
use std::collections::HashMap;
use std::path::PathBuf;
use glob::glob;
use crate::script::file::File;
use crate::script::path::{Path, PathRoot};
use crate::script::error::ScriptError;
use super::markdown;

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

        std::fs::create_dir_all(&self.output_path)?;

        std::fs::write(self.output_path.join("book.toml"), Self::default_mdbook_config())?;

        for script in &self.script_files {

            let output_path = self.get_output_path(&script.path);
            std::fs::create_dir_all(output_path.parent().unwrap())?;
            let mut file = std::fs::File::create(output_path)?;

            for (_, model) in &script.semantic.as_ref().unwrap().script.read().unwrap().models {
                file.write_all(markdown::model(&model.read().unwrap()).as_bytes())?;
            }

            for (_, sequence) in &script.semantic.as_ref().unwrap().script.read().unwrap().sequences {
                file.write_all(markdown::sequence(&sequence.read().unwrap()).as_bytes())?;
            }
        }

        Ok(())
    }

    fn get_output_path(&self, path: &Path) -> PathBuf {

        let mut os_path = self.output_path.join("src");

        for step in path.path() {
            os_path = os_path.join(step);
        }

        os_path.set_extension("md");

        os_path
    }

    fn default_mdbook_config() -> &'static str {
        r#"
        [book]
        authors = ["The Author"]
        language = "en"
        multilingual = false
        src = "src"
        title = "Documentation"
        "#
    }
}