
use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Model as TextModel;

use super::script::Script;
use super::reference::Reference;
use super::r#use::Use;

pub struct Model {
    pub text: TextModel,

    pub script: Rc<RefCell<Script>>,

    pub name: String,
    pub r#type: Reference<Use>,
}

impl Model {
    pub fn new(script: Rc<RefCell<Script>>, text: TextModel) -> Result<Rc<RefCell<Self>>, ScriptError> {

        {
            let borrowed_script = script.borrow();

            let model = borrowed_script.find_models(&text.name);
            if model.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &text.name+ "' is already declared."))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            script,
            name: text.name.clone(),
            r#type: Reference::new(text.r#type.clone()),
            text,
        })))
    }

    pub fn make_references(&mut self) -> Result<(), ScriptError> {

        let borrowed_script = self.script.borrow();

        let r#use = borrowed_script.find_use(&self.r#type.name);
        if r#use.is_none() {
            return Err(ScriptError::semantic("'".to_string() + &self.r#type.name + "' is unkown."))
        }

        self.r#type.reference = Some(Rc::clone(r#use.unwrap()));

        Ok(())
    }
}
