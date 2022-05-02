
//! Provides script paths management.

use std::fmt;
use crate::logic::descriptor::identifier::{Identifier, Root};

/// Container-helper structure for paths in scripts.
/// 
/// It is used for handling `use` paths, as well as representing paths up to elements to build identifiers.
#[derive(Clone, PartialEq, Debug)]
pub struct Path {
    /// Vector of string containing literally the path steps.
    path: Vec<String>,
    root: PathRoot,
}

/// Convenience enum for handling and identifying path root types.
/// 
/// All values are self-describing regarding Mélodium rules, except `Other`, which actually indicates any invalid roots.
/// In most (all?) cases, having `Other` should end in error.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PathRoot {
    Core,
    Std,
    Main,
    Local,
    Other,
}

impl fmt::Display for PathRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let string = match self {
            PathRoot::Core => "core",
            PathRoot::Std => "std",
            PathRoot::Main => "main",
            PathRoot::Local => "local",
            PathRoot::Other => "other",
        };

        write!(f, "{}", string)
    }
}

impl Path {
    /// Instanciates a new path.
    /// 
    /// ```
    /// # use melodium::script::path::{Path, PathRoot};
    /// // use main/foo/bar::UselessElement
    /// let raw_valid_path = vec![  "main".to_string(),
    ///                             "foo".to_string(),
    ///                             "bar".to_string()];
    /// 
    /// let valid_path = Path::new(raw_valid_path);
    /// assert_eq!(valid_path.root(), PathRoot::Main);
    /// assert!(valid_path.is_valid());
    /// 
    /// // use oops/i/made/typo::UnreachableElement
    /// let raw_invalid_path = vec!["oops".to_string(),
    ///                             "i".to_string(),
    ///                             "made".to_string(),
    ///                             "typo".to_string()];
    /// 
    /// let invalid_path = Path::new(raw_invalid_path);
    /// assert_eq!(invalid_path.root(), PathRoot::Other);
    /// assert!(!invalid_path.is_valid());
    /// ```
    pub fn new(path: Vec<String>) -> Self {

        let first = path.first();
        let root = if let Some(val) = first {
            match val.as_str() {
                "core" => PathRoot::Core,
                "std" => PathRoot::Std,
                "main" => PathRoot::Main,
                "local" => PathRoot::Local,
                _ => PathRoot::Other
            }
        }
        else {
            PathRoot::Other
        };

        Self {
            root,
            path: path.iter().skip(1).map(|s| s.clone()).collect()
        }
    }

    /// Gives immutable reference to vector of string containing literally the path steps.
    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn root(&self) -> PathRoot {
        self.root
    }

    /// Tells if the path is valid.
    /// 
    /// It is a shorthand for `path.root() != PathRoot::Other`.
    pub fn is_valid(&self) -> bool {
        self.root != PathRoot::Other
    }

    /// Turn the path into an identifier.
    /// 
    /// * `element_name`: name of the element supposed to be identified under that path.
    /// 
    /// # Warning
    /// A path can only be turned into identifier if its root is `core`/[PathRoot::Core](super::path::PathRoot::Core), `std`/[PathRoot::Std](super::path::PathRoot::Std) or `main`/[PathRoot::Main](super::path::PathRoot::Main) (local paths are not absolute so not usable to make idenfier).
    /// The `core` conversion to identifier is available only for convenience, as nothing in scripted can become part of the core.
    pub fn to_identifier(&self, element_name: &str) -> Option<Identifier> {

        if let Some(root) = match self.root {
            PathRoot::Core => Some(Root::Core),
            PathRoot::Std => Some(Root::Std),
            PathRoot::Main => Some(Root::Main),
            _ => None
        } {
            Some(Identifier::new(root, self.path.clone(), element_name))
        }
        else {
            None
        }
        
    }
}
