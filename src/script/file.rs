
use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use super::path::Path;
use crate::script::error::ScriptError;
use crate::script::text::Script as TextScript;
use crate::script::semantic::common::Tree;

pub struct File {
    pub absolute_path: PathBuf,
    /// Canonical path inside Mélodium.
    /// 
    /// May start either by "std" or "main", but not "local", as it is then a relative path, that have to be translated into a canonical one.
    pub path: Path,
    pub text: Option<String>,
    pub semantic: Option<Tree>,
}

impl File {
    pub fn new<P: Into<PathBuf>>(path: Path, absolute_path: P) -> Self {
        Self {
            absolute_path: absolute_path.into(),
            path,
            text: None,
            semantic: None,
        }
    }

    pub fn read(&mut self) -> io::Result<()> {

        let mut file = fs::File::open(&self.absolute_path)?;

        let mut text = String::new();
        file.read_to_string(&mut text)?;

        self.text = Some(text);

        Ok(())
    }

    pub fn parse(&mut self) -> Result<(), ScriptError> {

        let parsed_text = TextScript::build(self.text.as_ref().unwrap())?;

        let semantic_tree = Tree::new(parsed_text)?;
        semantic_tree.make_references()?;

        self.semantic = Some(semantic_tree);

        Ok(())
    }
}
