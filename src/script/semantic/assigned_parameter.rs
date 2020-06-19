
use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::treatment::Treatment;
use super::value::Value;

pub struct AssignedParameter {
    pub text: TextParameter,

    pub treatment: Rc<RefCell<Treatment>>,

    pub name: String,
    pub value: Rc<RefCell<Value>>,
}

impl AssignedParameter {
    pub fn new(treatment: Rc<RefCell<Treatment>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let value;
        {
            let borrowed_treatment = treatment.borrow();

            let parameter = borrowed_treatment.find_parameter(&text.name);
            if parameter.is_some() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' is already assigned."))
            }

            if text.value.is_some() {
                value = Value::new(Rc::clone(&borrowed_treatment.sequence), text.value.as_ref().unwrap().clone())?;
            }
            else {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' is missing value."))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            name: text.name.clone(),
            text,
            treatment,
            value,
        })))
    }

    pub fn make_references(&self) -> Result<(), ScriptError> {

        self.value.borrow_mut().make_references()?;

        Ok(())
    }

}