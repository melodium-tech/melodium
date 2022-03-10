
//! Provides script files management.

use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use super::path::Path;
use crate::script::error::ScriptError;
use crate::script::text::Script as TextScript;
use crate::script::semantic::common::Tree;

/// Manage script file.
/// 
/// Handle a system path and Mélodium path, generate and holds script textual and semantic content.
pub struct File {
    /// Absolute system path.
    /// 
    /// This path should be absolute in order to not have duplicates parsing and semantic processing of the same content.
    pub absolute_path: PathBuf,
    /// Canonical path inside Mélodium.
    /// 
    /// May start either by `std`/[PathRoot::Std](super::path::PathRoot::Std) or `main`/[PathRoot::Main](super::path::PathRoot::Main), but not `local`/[PathRoot::Local](super::path::PathRoot::Local), as it is then a relative path, that have to be translated into a canonical one.
    pub path: Path,
    /// The whole textual content, if existing.
    pub text: Option<String>,
    /// The semantic tree, if built.
    pub semantic: Option<Tree>,
}

impl File {
    /// Instanciates a new Mélodium script file.
    /// 
    /// * `path`: canonical path inside Mélodium, may start either by `std`/[PathRoot::Std](super::path::PathRoot::Std) or `main`/[PathRoot::Main](super::path::PathRoot::Main), but not `local`/[PathRoot::Local](super::path::PathRoot::Local), as it is then a relative path, that have to be translated into a canonical one.
    /// * `absolute_path`: absolute system path, this path should be absolute in order to not have duplicates parsing and semantic processing of the same content.
    /// 
    /// This does not open nor even test if file exists, see `read()` and `parse()` methods.
    /// ```
    /// # use melodium::script::file::File;
    /// # use melodium::script::path::Path;
    /// # use std::path::PathBuf;
    /// // main/simple_build
    /// let path = Path::new(vec!["main".to_string(), "simple_build".to_string()]);
    /// 
    /// let relative_path = "examples/semantic/simple_build.mel";
    /// let absolute_path = PathBuf::from(relative_path).canonicalize().unwrap();
    /// 
    /// let file = File::new(path, absolute_path);
    /// 
    /// assert!(file.text.is_none());
    /// assert!(file.semantic.is_none());
    /// ```
    pub fn new<P: Into<PathBuf>>(path: Path, absolute_path: P) -> Self {
        Self {
            absolute_path: absolute_path.into(),
            path,
            text: None,
            semantic: None,
        }
    }

    /// Reads and loads the file content.
    /// 
    /// This method open and close the file. In other words, the script file is only opened during the call time of this method.
    /// Any read error is reported through the result return value.
    /// 
    /// ```
    /// # use melodium::script::file::File;
    /// # use melodium::script::path::Path;
    /// # use std::path::PathBuf;
    /// # use std::io::Error;
    /// // main/simple_build
    /// let path = Path::new(vec!["main".to_string(), "simple_build".to_string()]);
    /// 
    /// let relative_path = "examples/semantic/simple_build.mel";
    /// let absolute_path = PathBuf::from(relative_path).canonicalize().unwrap();
    /// 
    /// let mut file = File::new(path, absolute_path);
    /// 
    /// file.read()?;
    /// 
    /// assert!(file.text.is_some());
    /// assert!(file.semantic.is_none());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn read(&mut self) -> io::Result<()> {

        let mut file = fs::File::open(&self.absolute_path)?;

        let mut text = String::new();
        file.read_to_string(&mut text)?;

        self.text = Some(text);

        Ok(())
    }

    /// Parse the file content.
    /// 
    /// This method have to be used after a successful call on `read()`. It makes the parsing of `text` and builds the semantic tree.
    /// 
    /// ```
    /// # use melodium::script::file::File;
    /// # use melodium::script::path::Path;
    /// # use std::path::PathBuf;
    /// # use melodium::script::error::ScriptError;
    /// // main/simple_build
    /// let path = Path::new(vec!["main".to_string(), "simple_build".to_string()]);
    /// 
    /// let relative_path = "examples/semantic/simple_build.mel";
    /// let absolute_path = PathBuf::from(relative_path).canonicalize().unwrap();
    /// 
    /// let mut file = File::new(path, absolute_path);
    /// 
    /// file.read();
    /// file.parse()?;
    /// 
    /// assert!(file.text.is_some());
    /// assert!(file.semantic.is_some());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn parse(&mut self) -> Result<(), ScriptError> {

        let parsed_text = TextScript::build(self.text.as_ref().unwrap())?;

        let semantic_tree = Tree::new(parsed_text)?;
        semantic_tree.make_references(&self.path)?;

        self.semantic = Some(semantic_tree);

        Ok(())
    }
}
