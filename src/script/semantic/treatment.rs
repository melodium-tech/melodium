
use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Treatment as TextTreatment;

use super::r#use::Use;
use super::sequence::Sequence;
use super::common::Reference;
use super::assigned_parameter::AssignedParameter;

pub struct Treatment {
    pub text: TextTreatment,

    pub sequence: Rc<RefCell<Sequence>>,

    pub name: String,
    pub r#type: RefersTo,
    pub parameters: Vec<Rc<RefCell<AssignedParameter>>>,
}

pub enum RefersTo {
    Unkown(Reference<()>),
    Use(Reference<Use>),
    Sequence(Reference<Sequence>),
}

impl Treatment {
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextTreatment) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let treatment = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            text: text.clone(),
            sequence: Rc::clone(&sequence),
            name: text.name.clone(),
            r#type: RefersTo::Unkown(Reference::new(text.r#type)),
            parameters: Vec::new(),
        }));

        {
            let borrowed_sequence = sequence.borrow();

            let treatment = borrowed_sequence.find_treatment(&text.name);
            if treatment.is_some() {
                return Err(ScriptError::semantic("Treatment '".to_string() + &text.name + "' is already declared."))
            }
        }

        for p in text.parameters {
            let assigned_parameter = AssignedParameter::new(Rc::clone(&treatment), p)?;
            treatment.borrow_mut().parameters.push(assigned_parameter);
        }

        Ok(treatment)
    }

    pub fn find_parameter(&self, name: & str) -> Option<&Rc<RefCell<AssignedParameter>>> {
        self.parameters.iter().find(|&p| p.borrow().name == name) 
    }
}

impl Node for Treatment {
    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {

        let mut children: Vec<Rc<RefCell<dyn Node>>> = Vec::new();

        self.parameters.iter().for_each(|p| children.push(Rc::clone(&p) as Rc<RefCell<dyn Node>>));

        children
    }

    fn make_references(&mut self) -> Result<(), ScriptError> {

        if let RefersTo::Unkown(reference) = &self.r#type {

            let borrowed_sequence = self.sequence.borrow();
            let borrowed_script = borrowed_sequence.script.borrow();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {

                self.r#type = RefersTo::Use(Reference{
                    name: reference.name.clone(),
                    reference: Some(Rc::clone(r#use.unwrap()))
                });
            }
            else {
                let sequence = borrowed_script.find_sequence(&reference.name);
                if sequence.is_some() {

                    self.r#type = RefersTo::Sequence(Reference{
                        name: reference.name.clone(),
                        reference: Some(Rc::clone(sequence.unwrap()))
                    });
                }
                else {
                    return Err(ScriptError::semantic("'".to_string() + &reference.name + "' is unkown."))
                }
            }
        }

        Ok(())
    }
}
