//! Provides script paths management.

use melodium_common::descriptor::{Identifier, IdentifierRequirement, Version};

/// Container-helper structure for paths in scripts.
///
/// It is used for handling `use` paths, as well as representing paths up to elements to build identifiers.
#[derive(Clone, PartialEq, Debug)]
pub struct Path {
    /// Version of the elements located at path
    version: Version,
    /// Vector of string containing literally the path steps.
    path: Vec<String>,
}

impl Path {
    /// Instanciates a new path.
    ///
    /// ```
    /// # use melodium_lang::Path;
    /// // use main/foo/bar::Element
    /// let raw_valid_path = vec![  "main".to_string(),
    ///                             "foo".to_string(),
    ///                             "bar".to_string()];
    ///
    /// let valid_path = Path::new(raw_valid_path);
    /// assert_eq!(valid_path.root(), "main");
    /// assert!(valid_path.is_valid());
    /// ```
    pub fn new(version: Version, path: Vec<String>) -> Self {
        Self { version, path }
    }

    /// Version used for the path.
    pub fn version(&self) -> &Version {
        &self.version
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
        } else {
            false
        }
    }

    /// Turn the path into an identifier.
    ///
    /// * `element_name`: name of the element supposed to be identified under that path.
    ///
    /// # Warning
    /// Return `None` if the path is invalid.
    pub fn to_identifier(&self, element_name: &str) -> Option<Identifier> {
        if self.is_valid() {
            Some(Identifier::new_versionned(
                &self.version,
                self.path.clone(),
                element_name,
            ))
        } else {
            None
        }
    }

    pub fn to_identifier_requirement(&self, element_name: &str) -> Option<IdentifierRequirement> {
        if self.is_valid() {
            Some(
                (&Identifier::new_versionned(&self.version, self.path.clone(), element_name))
                    .into(),
            )
        } else {
            None
        }
    }
}
