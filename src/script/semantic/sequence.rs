
//! Module dedicated to Sequence semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Sequence as TextSequence;

use super::script::Script;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::declared_parameter::DeclaredParameter;
use super::requirement::Requirement;
use super::input::Input;
use super::output::Output;
use super::treatment::Treatment;
use super::connection::Connection;

/// Structure managing and describing semantic of a sequence.
/// 
/// It owns the whole [text sequence](../../text/sequence/struct.Sequence.html).
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
    /// Create a new semantic sequence, based on textual sequence.
    /// 
    /// * `script`: the parent script that "owns" this sequence.
    /// * `text`: the textual sequence.
    /// 
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](../common/trait.Node.html).
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Sequence::new(Rc::clone(&script), text_sequence)
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("PrepareAudioFiles").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_sequence.parameters.len(), 5);
    /// assert_eq!(borrowed_sequence.treatments.len(), 2);
    /// assert_eq!(borrowed_sequence.origin.as_ref().unwrap().borrow().name, "AudioFiles");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(script: Rc<RefCell<Script>>, text: TextSequence) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let sequence = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            text: text.clone(),
            script: Rc::clone(&script),
            name: text.name.string.clone(),
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

            let sequence = borrowed_script.find_sequence(&text.name.string);
            if sequence.is_some() {
                return Err(ScriptError::semantic("Sequence '".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }

            let r#use = borrowed_script.find_use(&text.name.string);
            if r#use.is_some() {
                return Err(ScriptError::semantic("Element '".to_string() + &text.name.string + "' is already declared as used.", text.name.position))
            }
        }

        for p in text.parameters {
            let declared_parameter = DeclaredParameter::new(Rc::clone(&sequence) as Rc<RefCell<dyn DeclarativeElement>>, p)?;
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

    /// Search for a requirement.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("Main").unwrap().borrow();
    /// 
    /// let signal = borrowed_sequence.find_requirement("@Signal");
    /// let dont_exist = borrowed_sequence.find_requirement("@dontExist");
    /// assert!(signal.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_requirement(&self, name: & str) -> Option<&Rc<RefCell<Requirement>>> {
        self.requirements.iter().find(|&r| r.borrow().name == name) 
    }

    /// Search for an input.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("MakeHPCP").unwrap().borrow();
    /// 
    /// let spectrum = borrowed_sequence.find_input("spectrum");
    /// let dont_exist = borrowed_sequence.find_input("dontExist");
    /// assert!(spectrum.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_input(&self, name: & str) -> Option<&Rc<RefCell<Input>>> {
        self.inputs.iter().find(|&i| i.borrow().name == name) 
    }

    /// Search for an output.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("MakeHPCP").unwrap().borrow();
    /// 
    /// let hpcp = borrowed_sequence.find_output("hpcp");
    /// let dont_exist = borrowed_sequence.find_output("dontExist");
    /// assert!(hpcp.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_output(&self, name: & str) -> Option<&Rc<RefCell<Output>>> {
        self.outputs.iter().find(|&o| o.borrow().name == name) 
    }

    /// Search for a treatment.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("MakeSpectrum").unwrap().borrow();
    /// 
    /// let frame_cutter = borrowed_sequence.find_treatment("FrameCutter");
    /// let dont_exist = borrowed_sequence.find_treatment("dontExist");
    /// assert!(frame_cutter.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_treatment(&self, name: & str) -> Option<&Rc<RefCell<Treatment>>> {
        self.treatments.iter().find(|&t| t.borrow().name == name) 
    }
}

impl Node for Sequence {
    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {

        let mut children: Vec<Rc<RefCell<dyn Node>>> = Vec::new();

        self.parameters.iter().for_each(|p| children.push(Rc::clone(&p) as Rc<RefCell<dyn Node>>));
        self.requirements.iter().for_each(|r| children.push(Rc::clone(&r) as Rc<RefCell<dyn Node>>));
        self.inputs.iter().for_each(|i| children.push(Rc::clone(&i) as Rc<RefCell<dyn Node>>));
        self.outputs.iter().for_each(|o| children.push(Rc::clone(&o) as Rc<RefCell<dyn Node>>));
        self.treatments.iter().for_each(|t| children.push(Rc::clone(&t) as Rc<RefCell<dyn Node>>));
        self.connections.iter().for_each(|c| children.push(Rc::clone(&c) as Rc<RefCell<dyn Node>>));

        children
    }
}

impl DeclarativeElement for Sequence {

    fn declarative_element(&self) -> DeclarativeElementType {
        DeclarativeElementType::Sequence(&self)
    }

    /// Search for a parameter.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// # use melodium_rust::script::semantic::declarative_element::DeclarativeElement;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("PrepareAudioFiles").unwrap().borrow();
    /// 
    /// let sample_rate = borrowed_sequence.find_declared_parameter("sampleRate");
    /// let dont_exist = borrowed_sequence.find_declared_parameter("dontExist");
    /// assert!(sample_rate.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_declared_parameter(&self, name: & str) -> Option<&Rc<RefCell<DeclaredParameter>>> {
        self.parameters.iter().find(|&p| p.borrow().name == name)
    }
}
