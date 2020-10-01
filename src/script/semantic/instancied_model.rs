
use super::common::Node;

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Instanciation as TextInstanciation;

use super::r#use::Use;
use super::model::Model;
use super::sequence::Sequence;
use super::common::Reference;
use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_parameter::AssignedParameter;
use super::declarative_element::DeclarativeElement;


pub struct InstanciedModel {
    pub text: TextInstanciation,

    pub sequence: Weak<RefCell<Sequence>>,

    pub name: String,
    pub r#type: RefersTo,
    pub parameters: Vec<Rc<RefCell<AssignedParameter>>>,
}

pub enum RefersTo {
    Unkown(Reference<()>),
    Use(Reference<Use>),
    Model(Reference<Model>),
}

impl InstanciedModel {
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextInstanciation) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let treatment = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            text: text.clone(),
            sequence: Rc::downgrade(&sequence),
            name: text.name.string.clone(),
            r#type: RefersTo::Unkown(Reference::new(text.r#type.string)),
            parameters: Vec::new(),
        }));

        {
            let borrowed_sequence = sequence.borrow();

            let treatment = borrowed_sequence.find_instancied_model(&text.name.string);
            if treatment.is_some() {
                return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' is already instancied.", text.name.position))
            }
        }

        for p in text.parameters {
            let assigned_parameter = AssignedParameter::new(Rc::clone(&treatment) as Rc<RefCell<dyn AssignativeElement>>, p)?;
            treatment.borrow_mut().parameters.push(assigned_parameter);
        }

        Ok(treatment)
    }
}

impl AssignativeElement for InstanciedModel {

    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::InstanciedModel(&self)
    }

    fn associated_declarative_element(&self) -> Rc<RefCell<dyn DeclarativeElement>> {
        self.sequence.upgrade().unwrap() as Rc<RefCell<dyn DeclarativeElement>>
    }

    fn find_assigned_parameter(&self, name: & str) -> Option<&Rc<RefCell<AssignedParameter>>> {
        self.parameters.iter().find(|&a| a.borrow().name == name)
    }
}

impl Node for InstanciedModel {
    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {

        let mut children: Vec<Rc<RefCell<dyn Node>>> = Vec::new();

        self.parameters.iter().for_each(|p| children.push(Rc::clone(&p) as Rc<RefCell<dyn Node>>));

        children
    }

    fn make_references(&mut self) -> Result<(), ScriptError> {

        if let RefersTo::Unkown(reference) = &self.r#type {

            let rc_sequence = self.sequence.upgrade().unwrap();
            let borrowed_sequence = rc_sequence.borrow();
            let rc_script = borrowed_sequence.script.upgrade().unwrap();
            let borrowed_script = rc_script.borrow();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {

                self.r#type = RefersTo::Use(Reference{
                    name: reference.name.clone(),
                    reference: Some(Rc::downgrade(r#use.unwrap()))
                });
            }
            else {
                let model = borrowed_script.find_model(&reference.name);
                if model.is_some() {

                    self.r#type = RefersTo::Model(Reference{
                        name: reference.name.clone(),
                        reference: Some(Rc::downgrade(model.unwrap()))
                    });
                }
                else {
                    return Err(ScriptError::semantic("'".to_string() + &reference.name + "' is unkown.", self.text.r#type.position))
                }
            }
        }

        Ok(())
    }
}
