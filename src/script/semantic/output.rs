
use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::sequence::Sequence;
use super::r#type::Type;

pub struct Output {
    pub text: TextParameter,

    pub sequence: Rc<RefCell<Sequence>>,

    pub name: String,
    pub r#type: Type,
}

impl Output {
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let r#type;
        {
            let borrowed_sequence = sequence.borrow();

            let input = borrowed_sequence.find_output(&text.name);
            if input.is_some() {
                return Err(ScriptError::semantic("Output '".to_string() + &text.name + "' is already declared."))
            }

            if text.r#type.is_none() {
                return Err(ScriptError::semantic("Output '".to_string() + &text.name + "' do not have type."))
            }
            r#type = Type::new(text.r#type.as_ref().unwrap().clone())?;

            if text.value.is_some() {
                return Err(ScriptError::semantic("Output '".to_string() + &text.name + "' cannot have default value."))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self{
            sequence,
            name: text.name.clone(),
            text,
            r#type,
        })))
    }
}

