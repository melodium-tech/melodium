
use super::SemanticNode;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::sequence::Sequence;
use super::r#type::Type;
use super::value::Value;

pub struct DeclaredParameter {
    pub text: TextParameter,

    pub sequence: Rc<RefCell<Sequence>>,

    pub name: String,
    pub r#type: Type,
    pub value: Option<Rc<RefCell<Value>>>,
}

impl DeclaredParameter {
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let r#type;
        let value;
        {
            let borrowed_sequence = sequence.borrow();

            let parameter = borrowed_sequence.find_parameter(&text.name);
            if parameter.is_some() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' is already declared."))
            }

            if text.r#type.is_none() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' do not have type."))
            }
            r#type = Type::new(text.r#type.as_ref().unwrap().clone())?;

            if text.value.is_some() {
                value = Some(Value::new(Rc::clone(&sequence), text.value.as_ref().unwrap().clone())?);
            }
            else {
                value = None;
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            sequence,
            name: text.name.clone(),
            text,
            r#type,
            value,
        })))
    }
}

impl SemanticNode for DeclaredParameter {
    
}

