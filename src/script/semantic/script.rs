

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Script as TextScript;

use super::r#use::Use;
use super::model::Model;
use super::sequence::Sequence;

pub struct Script {
    pub text: TextScript,

    pub address: String,

    pub uses: Vec<Rc<RefCell<Use>>>,
    pub models: Vec<Rc<RefCell<Model>>>,
    pub sequences: Vec<Rc<RefCell<Sequence>>>,
}

impl Script {
    pub fn new(address: & str, text: TextScript) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let script = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            text: text.clone(),
            address: address.to_string(),
            uses: Vec::new(),
            models: Vec::new(),
            sequences: Vec::new(),
        }));

        for u in text.uses {
            let r#use = Use::new(Rc::clone(&script), u.clone())?;
            script.borrow_mut().uses.push(r#use);
        }

        for m in text.models {
            let model = Model::new(Rc::clone(&script), m.clone())?;
            script.borrow_mut().models.push(model);
        }

        for s in text.sequences {
            let sequence = Sequence::new(Rc::clone(&script), s.clone())?;
            script.borrow_mut().sequences.push(sequence);
        }

        Ok(script)
    }

    pub fn find_use(&self, element: & str) -> Option<&Rc<RefCell<Use>>> {
        self.uses.iter().find(|&u| u.borrow().element == element)
    }

    pub fn find_models(&self, name: & str) -> Option<&Rc<RefCell<Model>>> {
        self.models.iter().find(|&m| m.borrow().name == name)
    }

    pub fn find_sequence(&self, name: & str) -> Option<&Rc<RefCell<Sequence>>> {
        self.sequences.iter().find(|&s| s.borrow().name == name)
    }
}

impl Node for Script {
    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {

        let mut children: Vec<Rc<RefCell<dyn Node>>> = Vec::new();

        self.uses.iter().for_each(|u| children.push(Rc::clone(&u) as Rc<RefCell<dyn Node>>));
        self.models.iter().for_each(|m| children.push(Rc::clone(&m) as Rc<RefCell<dyn Node>>));
        self.sequences.iter().for_each(|s| children.push(Rc::clone(&s) as Rc<RefCell<dyn Node>>));

        children
    }
}

#[cfg(test)]
mod tests {

    use super::Node;
    use crate::script::semantic::common::Tree;
    use crate::script_file::ScriptFile;

    #[test]
    fn test_simple_semantic() {

        let address = "examples/semantic/simple_build.mel";

        let mut script_file = ScriptFile::new(address);

        script_file.load().unwrap();
        script_file.parse().unwrap();

        let semantic_tree = Tree::new(address, script_file.script().clone());
        semantic_tree.make_references().unwrap();

        assert_eq!(semantic_tree.script.borrow().sequences.len(), 4);
    }
}