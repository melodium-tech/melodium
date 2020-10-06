
//! Module dedicated to Model semantic analysis.

use super::common::Node;

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Model as TextModel;

use super::script::Script;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::declared_parameter::DeclaredParameter;
use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_parameter::AssignedParameter;
use super::common::Reference;
use super::r#use::Use;

/// Structure managing and describing semantic of a model.
/// 
/// It owns the whole [text model](../../text/model/struct.Model.html).
pub struct Model {
    pub text: TextModel,

    pub script: Weak<RefCell<Script>>,

    pub name: String,
    pub parameters: Vec<Rc<RefCell<DeclaredParameter>>>,
    pub r#type: Reference<Use>,
    pub assignations: Vec<Rc<RefCell<AssignedParameter>>>,

    auto_reference: Weak<RefCell<Self>>,
}

impl Model {
    /// Create a new semantic model, based on textual model.
    /// 
    /// * `script`: the parent script that "owns" this model.
    /// * `text`: the textual model.
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
    /// // Internally, Script::new call Model::new(Rc::clone(&script), text_model)
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_model = borrowed_script.find_model("Files").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_model.parameters.len(), 1);
    /// assert_eq!(borrowed_model.assignations.len(), 1);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(script: Rc<RefCell<Script>>, text: TextModel) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let model = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            text: text.clone(),
            script: Rc::downgrade(&script),
            name: text.name.string.clone(),
            parameters: Vec::new(),
            r#type: Reference::new(text.r#type.string.clone()),
            assignations: Vec::new(),
            auto_reference: Weak::new(),
        }));

        model.borrow_mut().auto_reference = Rc::downgrade(&model);

        {
            let borrowed_script = script.borrow();

            let model = borrowed_script.find_model(&text.name.string);
            if model.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }
        }

        for p in text.parameters {
            let declared_parameter = DeclaredParameter::new(Rc::clone(&model) as Rc<RefCell<dyn DeclarativeElement>>, p)?;
            model.borrow_mut().parameters.push(declared_parameter);
        }

        for a in text.assignations {
            let assigned_parameter = AssignedParameter::new(Rc::clone(&model) as Rc<RefCell<dyn AssignativeElement>>, a)?;
            model.borrow_mut().assignations.push(assigned_parameter);
        }

        Ok(model)
    }
}

impl DeclarativeElement for Model {
    
    fn declarative_element(&self) -> DeclarativeElementType {
        DeclarativeElementType::Model(&self)
    }

    /// Search for a declared parameter.
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
    /// let borrowed_model = borrowed_script.find_model("Files").unwrap().borrow();
    /// 
    /// let directory = borrowed_model.find_declared_parameter("directory");
    /// let dont_exist = borrowed_model.find_declared_parameter("dontExist");
    /// assert!(directory.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_declared_parameter(&self, name: & str) -> Option<&Rc<RefCell<DeclaredParameter>>> {
        self.parameters.iter().find(|&p| p.borrow().name == name)
    }

}

impl AssignativeElement for Model {
    
    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::Model(&self)
    }

    fn associated_declarative_element(&self) -> Rc<RefCell<dyn DeclarativeElement>> {
        self.auto_reference.upgrade().unwrap()
    }

    /// Search for an assigned parameter.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// # use melodium_rust::script::semantic::assignative_element::AssignativeElement;
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
    /// let borrowed_model = borrowed_script.find_model("AudioEngine").unwrap().borrow();
    /// 
    /// let sample_rate = borrowed_model.find_assigned_parameter("sampleRate");
    /// let dont_exist = borrowed_model.find_assigned_parameter("dontExist");
    /// assert!(sample_rate.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_assigned_parameter(&self, name: & str) -> Option<&Rc<RefCell<AssignedParameter>>> {
        self.assignations.iter().find(|&a| a.borrow().name == name)
    }

}

impl Node for Model {
    
    fn make_references(&mut self) -> Result<(), ScriptError> {

        let rc_script = self.script.upgrade().unwrap();
        let borrowed_script = rc_script.borrow();

        let r#use = borrowed_script.find_use(&self.r#type.name);
        if r#use.is_none() {
            return Err(ScriptError::semantic("'".to_string() + &self.r#type.name + "' is unkown.", self.text.r#type.position))
        }

        self.r#type.reference = Some(Rc::downgrade(r#use.unwrap()));

        Ok(())
    }
}
