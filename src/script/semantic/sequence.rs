
use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Sequence as TextSequence;

use super::script::Script;
use super::declared_parameter::DeclaredParameter;
use super::requirement::Requirement;
use super::input::Input;
use super::output::Output;
use super::treatment::Treatment;
use super::connection::Connection;

pub struct Sequence {
    pub text: TextSequence,

    pub script: Rc<RefCell<Script>>,

    pub name: String,

    pub parameters: Vec<Rc<RefCell<DeclaredParameter>>>,
    pub requirements: Vec<Rc<RefCell<Requirement>>>,
    pub origin: Option<Rc<RefCell<Treatment>>>,
    pub inputs: Vec<Rc<RefCell<Input>>>,
    pub outputs: Vec<Rc<RefCell<Output>>>,
    pub treatments: Vec<Rc<RefCell<Treatment>>>,
    pub connections: Vec<Rc<RefCell<Connection>>>
}

impl Sequence {
    pub fn new(script: Rc<RefCell<Script>>, text: TextSequence) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let sequence = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            text: text.clone(),
            script: Rc::clone(&script),
            name: text.name.clone(),
            parameters: Vec::new(),
            requirements: Vec::new(),
            origin: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            treatments: Vec::new(),
            connections: Vec::new(),
        }));

        {
            let borrowed_script = script.borrow();

            let sequence = borrowed_script.find_sequence(&text.name);
            if sequence.is_some() {
                return Err(ScriptError::semantic("Sequence '".to_string() + &text.name + "' is already declared."))
            }

            let r#use = borrowed_script.find_use(&text.name);
            if r#use.is_some() {
                return Err(ScriptError::semantic("Element '".to_string() + &text.name + "' is already declared as used."))
            }
        }

        {
            let mut borrowed_sequence = sequence.borrow_mut();

            for p in text.parameters {
                borrowed_sequence.parameters.push(DeclaredParameter::new(Rc::clone(&sequence), p)?);
            }

            for r in text.requirements {
                borrowed_sequence.requirements.push(Requirement::new(Rc::clone(&sequence), r)?);
            }

            if text.origin.is_some() {
                
                let origin = Treatment::new(Rc::clone(&sequence), text.origin.unwrap())?;
                borrowed_sequence.origin = Some(Rc::clone(&origin));
                borrowed_sequence.treatments.push(Rc::clone(&origin));
            }

            for i in text.inputs {
                borrowed_sequence.inputs.push(Input::new(Rc::clone(&sequence), i)?);
            }

            for o in text.outputs {
                borrowed_sequence.outputs.push(Output::new(Rc::clone(&sequence), o)?);
            }

            for t in text.treatments {
                borrowed_sequence.treatments.push(Treatment::new(Rc::clone(&sequence), t)?);
            }

            for c in text.connections {
                borrowed_sequence.connections.push(Connection::new(Rc::clone(&sequence), c)?);
            }
        }

        Ok(sequence)
    }

    pub fn make_references(&self) -> Result<(), ScriptError> {

        if self.origin.is_some() {
            self.origin.as_ref().unwrap().borrow_mut().make_references()?;
        }

        for t in &self.treatments {
            t.borrow_mut().make_references()?;
        }

        for c in &self.connections {
            c.borrow_mut().make_references()?;
        }

        Ok(())
    }

    pub fn find_parameter(&self, name: & str) -> Option<&Rc<RefCell<DeclaredParameter>>> {
        self.parameters.iter().find(|&p| p.borrow().name == name)
    }

    pub fn find_requirement(&self, name: & str) -> Option<&Rc<RefCell<Requirement>>> {
        self.requirements.iter().find(|&r| r.borrow().name == name) 
    }

    pub fn find_input(&self, name: & str) -> Option<&Rc<RefCell<Input>>> {
        self.inputs.iter().find(|&i| i.borrow().name == name) 
    }

    pub fn find_output(&self, name: & str) -> Option<&Rc<RefCell<Output>>> {
        self.outputs.iter().find(|&o| o.borrow().name == name) 
    }

    pub fn find_treatment(&self, name: & str) -> Option<&Rc<RefCell<Treatment>>> {
        self.treatments.iter().find(|&t| t.borrow().name == name) 
    }
}
