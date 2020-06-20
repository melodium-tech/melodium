
use super::SemanticNode;

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

        for p in text.parameters {
            let declared_parameter = DeclaredParameter::new(Rc::clone(&sequence), p)?;
            sequence.borrow_mut().parameters.push(declared_parameter);
        }

        for r in text.requirements {
            let requirement = Requirement::new(Rc::clone(&sequence), r)?;
            sequence.borrow_mut().requirements.push(requirement);
        }

        if text.origin.is_some() {

            let origin = Treatment::new(Rc::clone(&sequence), text.origin.unwrap())?;

            let mut borrowed_sequence = sequence.borrow_mut();
            borrowed_sequence.origin = Some(Rc::clone(&origin));
            borrowed_sequence.treatments.push(Rc::clone(&origin));
        }

        for i in text.inputs {
            let input = Input::new(Rc::clone(&sequence), i)?;
            sequence.borrow_mut().inputs.push(input);
        }

        for o in text.outputs {
            let output = Output::new(Rc::clone(&sequence), o)?;
            sequence.borrow_mut().outputs.push(output);
        }

        for t in text.treatments {
            let treatment = Treatment::new(Rc::clone(&sequence), t)?;
            sequence.borrow_mut().treatments.push(treatment);
        }

        for c in text.connections {
            let connection = Connection::new(Rc::clone(&sequence), c)?;
            sequence.borrow_mut().connections.push(connection);
        }

        Ok(sequence)
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

impl SemanticNode for Sequence {
    fn children(&self) -> Vec<Rc<RefCell<dyn SemanticNode>>> {

        let mut children: Vec<Rc<RefCell<dyn SemanticNode>>> = Vec::new();

        self.parameters.iter().for_each(|p| children.push(Rc::clone(&p) as Rc<RefCell<dyn SemanticNode>>));
        self.requirements.iter().for_each(|r| children.push(Rc::clone(&r) as Rc<RefCell<dyn SemanticNode>>));
        self.inputs.iter().for_each(|i| children.push(Rc::clone(&i) as Rc<RefCell<dyn SemanticNode>>));
        self.outputs.iter().for_each(|o| children.push(Rc::clone(&o) as Rc<RefCell<dyn SemanticNode>>));
        self.treatments.iter().for_each(|t| children.push(Rc::clone(&t) as Rc<RefCell<dyn SemanticNode>>));
        self.connections.iter().for_each(|c| children.push(Rc::clone(&c) as Rc<RefCell<dyn SemanticNode>>));

        children
    }
}
