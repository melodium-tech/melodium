
//! Module dedicated to common semantic elements & traits.

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::script::Script;
use crate::script::text::Script as TextScript;
use crate::script::error::ScriptError;

/// Semantic tree.
/// 
/// Currently holds the root script, which itself owns all other elements.
pub struct Tree {
    pub script: Rc<RefCell<Script>>,
}

impl Tree {
    pub fn new(text: TextScript) -> Result<Self, ScriptError> {

        Ok(Self {
            script: Script::new(text)?
        })
    }

    pub fn make_references(&self) -> Result<(), ScriptError> {

        Self::make_references_node(Rc::clone(&self.script) as Rc<RefCell<dyn Node>>)?;

        Ok(())
    }

    fn make_references_node(node: Rc<RefCell<dyn Node>>) -> Result<(), ScriptError> {

        node.borrow_mut().make_references()?;

        let children = node.borrow().children();
        for child in children {
            Self::make_references_node(child)?;
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
    fn make_references(&mut self) -> Result<(), ScriptError> {
        Ok(())
    }

    /// Give a vector of all children the node have, whatever kind they can be.
    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {
        vec![]
    }
}

/// Structure holding name and weak-counted reference to another element.
#[derive(Default)]
pub struct Reference<T> {
    pub name: String,
    pub reference: Option<Weak<RefCell<T>>>,
}

impl<T> Reference<T> {
    pub fn new(name: String) -> Self {

        Self {
            name,
            reference: None,
        }
    }
}