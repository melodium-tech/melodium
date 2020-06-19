
use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Use as TextUse;

use super::script::Script;

pub struct Use {
    pub text: TextUse,

    pub script: Rc<RefCell<Script>>,

    pub path: Vec<String>,
    pub file_path: String,
    pub element: String,
}

impl Use {
    pub fn new(script: Rc<RefCell<Script>>, text: TextUse) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let file_path = text.path.join("/");

        {
            let borrowed_script = script.borrow();

            let r#use = borrowed_script.find_use(&text.element);
            if r#use.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &text.element + "' is already used."))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            script,
            path: text.path.clone(),
            file_path,
            element: text.element.clone(),
            text,
        })))
    }
}
