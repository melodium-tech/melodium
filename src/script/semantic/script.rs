
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

        {
            let mut borrowed_script = script.borrow_mut();

            for u in text.uses {
                borrowed_script.uses.push(Use::new(Rc::clone(&script), u.clone())?);
            }

            for m in text.models {
                borrowed_script.models.push(Model::new(Rc::clone(&script), m.clone())?);
            }

            for s in text.sequences {
                borrowed_script.sequences.push(Sequence::new(Rc::clone(&script), s.clone())?);
            }
        }

        Ok(script)
    }

    pub fn make_references(&self) -> Result<(), ScriptError> {

        for m in &self.models {
            m.borrow_mut().make_references()?;
        }

        for s in &self.sequences {
            s.borrow_mut().make_references()?;
        }

        Ok(())
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