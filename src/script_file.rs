
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use crate::script::text::script::Script;
use crate::script::error::ScriptError;

pub struct ScriptFile {
    contents: String,
    path: PathBuf,
    script: Option<Script>,
    file: Option<File>
}

impl ScriptFile {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            contents: String::new(),
            path: path.into(),
            script: None,
            file: None,
        }
    }

    pub fn load(&mut self) -> Result<(), std::io::Error> {

        self.file = Some(File::open(&self.path)?);

        self.file.as_ref().unwrap().read_to_string(&mut self.contents)?;

        Ok(())
    }

    pub fn parse(&mut self) -> Result<(), ScriptError> {

        self.script = Some(Script::build(&self.contents)?);

        Ok(())
    }

    pub fn script(&self) -> &Script {
        &self.script.as_ref().unwrap()
    }
}
