
//! Module dedicated to common semantic elements & traits.

use std::sync::{Arc, Weak, RwLock};
use super::script::Script;
use crate::script::text::Script as TextScript;
use crate::script::error::ScriptError;
use crate::script::path::Path;

/// Semantic tree.
/// 
/// Currently holds the root script, which itself owns all other elements.
pub struct Tree {
    pub script: Arc<RwLock<Script>>,
}

impl Tree {
    pub fn new(text: TextScript) -> Result<Self, ScriptError> {

        Ok(Self {
            script: Script::new(text)?
        })
    }

    pub fn make_references(&self, path: &Path) -> Result<(), ScriptError> {

        Self::make_references_node(Arc::clone(&self.script) as Arc<RwLock<dyn Node>>, path)?;

        Ok(())
    }

    fn make_references_node(node: Arc<RwLock<dyn Node>>, path: &Path) -> Result<(), ScriptError> {

        node.write().unwrap().make_references(path)?;

        let children = node.read().unwrap().children();
        for child in children {
            Self::make_references_node(child, path)?;
        }

        Ok(())
    }
}

/// Semantic node.
/// 
/// Allows making cross-references and create semantic relationships.
/// Any semantic element should implement this trait.
pub trait Node {
    /// Create references to the other elements the actual node relies on.
    /// 
    /// This exclude parent-child references, which are made when creating the elements.
    /// 
    /// * `path`: path to the current element, its root can ony be `std`/[PathRoot::Std](super::path::PathRoot::Std) or `main`/[PathRoot::Main](super::path::PathRoot::Main)
    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {
        Ok(())
    }

    /// Give a vector of all children the node have, whatever kind they can be.
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        vec![]
    }
}

/// Structure holding name and weak-counted reference to another element.
#[derive(Default)]
pub struct Reference<T> {
    pub name: String,
    pub reference: Option<Weak<RwLock<T>>>,
}

impl<T> Reference<T> {
    pub fn new(name: String) -> Self {

        Self {
            name,
            reference: None,
        }
    }
}