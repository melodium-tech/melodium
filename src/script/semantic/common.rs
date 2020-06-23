
use std::rc::Rc;
use std::cell::RefCell;
use super::script::Script;
use crate::script::text::Script as TextScript;
use crate::script::error::ScriptError;

pub struct Tree {
    pub script: Rc<RefCell<Script>>,
}

impl Tree {
    pub fn new(address: & str, text: TextScript) -> Self {

        Self {
            script: Script::new(address, text).unwrap()
        }
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

pub trait Node {
    fn make_references(&mut self) -> Result<(), ScriptError> {
        Ok(())
    }

    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {
        vec![]
    }
}

#[derive(Default)]
pub struct Reference<T> {
    pub name: String,
    pub reference: Option<Rc<RefCell<T>>>,
}

impl<T> Reference<T> {
    pub fn new(name: String) -> Self {

        Self {
            name,
            reference: None,
        }
    }
}