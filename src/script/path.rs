
//! Provides script paths management.

use crate::logic::descriptor::identifier::Identifier;

/// Container-helper structure for paths in scripts.
/// 
/// It is used for handling `use` paths, as well as representing paths up to elements to build identifiers.
#[derive(Clone, PartialEq, Debug)]
pub struct Path {
    /// Vector of string containing literally the path steps.
    path: Vec<String>,
}

impl Path {
    /// Instanciates a new path.
    /// 
    /// ```
    /// # use melodium::script::path::Path;
    /// // use main/foo/bar::UselessElement
    /// let raw_valid_path = vec![  "main".to_string(),
    ///                             "foo".to_string(),
    ///                             "bar".to_string()];
    /// 
    /// let valid_path = Path::new(raw_valid_path);
    /// assert_eq!(valid_path.root(), "main");
    /// assert!(valid_path.is_valid());
    /// ```
    pub fn new(path: Vec<String>) -> Self {

        Self {
            path
        }
    }

    /// Gives immutable reference to vector of string containing literally the path steps.
    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn root(&self) -> String {
        self.path.first().map(|s| s.clone()).unwrap_or_default()
    }

    /// Tells if the path is valid.
    /// 
    /// Currently check if at least a root is set up and no empty elements are present
    pub fn is_valid(&self) -> bool {
        if self.path.len() > 0 {
            !self.path.iter().any(|s| s.is_empty())
        }
        else {
            false
        }
    }

    /// Turn the path into an identifier.
    /// 
    /// * `element_name`: name of the element supposed to be identified under that path.
    /// 
    /// # Warning
    /// A path can only be turned into identifier if its root exists and is different from `local` (local paths are not absolute so not usable to make idenfier).
    pub fn to_identifier(&self, element_name: &str) -> Option<Identifier> {

        if self.is_valid() {
            Some(Identifier::new(self.path.clone(), element_name))
        }
        else {
            None
        }
        
    }
}
