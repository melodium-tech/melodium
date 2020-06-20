

pub mod assigned_parameter;
pub mod connection;
pub mod declared_parameter;
pub mod input;
pub mod model;
pub mod output;
pub mod reference;
pub mod requirement;
pub mod script;
pub mod sequence;
pub mod treatment;
pub mod r#type;
pub mod r#use;
pub mod value;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;

pub struct SemanticTree {
    script: Rc<RefCell<script::Script>>,
}

impl SemanticTree {
    pub fn new(address: & str, text: super::text::Script) -> Self {

        Self {
            script: script::Script::new(address, text).unwrap()
        }
    }

    fn make_references(&self) -> Result<(), ScriptError> {

        Self::make_references_node(Rc::clone(&self.script) as Rc<RefCell<dyn SemanticNode>>)?;

        Ok(())
    }

    fn make_references_node(node: Rc<RefCell<dyn SemanticNode>>) -> Result<(), ScriptError> {

        node.borrow_mut().make_references()?;

        let children = node.borrow().children();
        for child in children {
            Self::make_references_node(child)?;
        }

        Ok(())
    }
}

pub trait SemanticNode {
    fn make_references(&mut self) -> Result<(), ScriptError> {
        Ok(())
    }

    fn children(&self) -> Vec<Rc<RefCell<dyn SemanticNode>>> {
        vec![]
    }
}
