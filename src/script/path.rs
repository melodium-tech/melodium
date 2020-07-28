
//! Provides script paths management.

/// Container-helper structure for paths in scripts.
/// 
/// It is mainly used for handling `use` paths.
#[derive(Clone, PartialEq, Debug)]
pub struct Path {
    /// Vector of string containing literally the path steps.
    path: Vec<String>,
    root: PathRoot
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

impl Path {
    /// Instanciates a new path.
    /// 
    /// ```
    /// # use melodium_rust::script::path::{Path, PathRoot};
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
            path,
            root
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
}
