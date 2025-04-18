//! Module dedicated to common semantic elements & traits.

use super::script::Script;
use crate::path::Path;
use crate::text::Script as TextScript;
use crate::ScriptResult;
use melodium_common::descriptor::{Version, VersionReq};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Weak},
};

/// Semantic tree.
///
/// Currently holds the root script, which itself owns all other elements.
#[derive(Debug)]
pub struct Tree {
    pub script: Arc<RwLock<Script>>,
}

impl Tree {
    pub fn new(text: TextScript, version: Version) -> ScriptResult<Self> {
        Script::new(text, version).and_then(|script| ScriptResult::new_success(Self { script }))
    }

    pub fn make_references(
        &self,
        path: &Path,
        versions: &HashMap<String, VersionReq>,
    ) -> ScriptResult<()> {
        Self::make_references_node(
            Arc::clone(&self.script) as Arc<RwLock<dyn Node>>,
            path,
            versions,
        )
    }

    fn make_references_node(
        node: Arc<RwLock<dyn Node>>,
        path: &Path,
        versions: &HashMap<String, VersionReq>,
    ) -> ScriptResult<()> {
        let mut result = node.write().unwrap().make_references(path, versions);

        if result.is_success() {
            let children = node.read().unwrap().children();
            for child in children {
                result =
                    result.and_degrade_failure(Self::make_references_node(child, path, versions));
            }
        }

        result
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
    /// * `path`: path to the current element
    /// * `versions`: versions for roots
    fn make_references(
        &mut self,
        _path: &Path,
        _versions: &HashMap<String, VersionReq>,
    ) -> ScriptResult<()> {
        ScriptResult::new_success(())
    }

    /// Give a vector of all children the node have, whatever kind they can be.
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        vec![]
    }
}

/// Structure holding name and weak-counted reference to another element.
#[derive(Default, Debug, Clone)]
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
