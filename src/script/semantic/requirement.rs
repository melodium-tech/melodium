
use super::SemanticNode;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Requirement as TextRequirement;

use super::sequence::Sequence;

pub struct Requirement {
    pub text: TextRequirement,

    pub sequence: Rc<RefCell<Sequence>>,

    pub name: String,
}

impl Requirement {
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextRequirement) -> Result<Rc<RefCell<Self>>, ScriptError> {

        {
            let borrowed_sequence = sequence.borrow();

            let requirement = borrowed_sequence.find_requirement(&text.name);
            if requirement.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &text.name + "' is already required."))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self{
            sequence,
            name: text.name.clone(),
            text,
        })))
    }
}

impl SemanticNode for Requirement {
    
}
