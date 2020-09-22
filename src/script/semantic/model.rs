
//! Module dedicated to Model semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Model as TextModel;

use super::script::Script;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::declared_parameter::DeclaredParameter;
use super::common::Reference;
use super::r#use::Use;

/// Structure managing and describing semantic of a model.
/// 
/// It owns the whole [text model](../../text/model/struct.Model.html).
pub struct Model {
    pub text: TextModel,

    pub script: Rc<RefCell<Script>>,

    pub name: String,
    pub parameters: Vec<Rc<RefCell<DeclaredParameter>>>,
    pub r#type: Reference<Use>,
}

impl Model {
    /// Create a new semantic model, based on textual model.
    /// 
    /// * `script`: the parent script that "owns" this model.
    /// * `text`: the textual model.
    /// 
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](../common/trait.Node.html).
    pub fn new(script: Rc<RefCell<Script>>, text: TextModel) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let model = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            text: text.clone(),
            script: Rc::clone(&script),
            name: text.name.string.clone(),
            parameters: Vec::new(),
            r#type: Reference::new(text.r#type.string.clone()),
        }));

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

        Ok(model)
    }
}

impl DeclarativeElement for Model {
    
    fn declarative_element(&self) -> DeclarativeElementType {
        DeclarativeElementType::Model(&self)
    }

    /// Search for a declared parameter.
    fn find_declared_parameter(&self, name: & str) -> Option<&Rc<RefCell<DeclaredParameter>>> {
        self.parameters.iter().find(|&p| p.borrow().name == name)
    }

}

impl Node for Model {
    
    fn make_references(&mut self) -> Result<(), ScriptError> {

        let borrowed_script = self.script.borrow();

        let r#use = borrowed_script.find_use(&self.r#type.name);
        if r#use.is_none() {
            return Err(ScriptError::semantic("'".to_string() + &self.r#type.name + "' is unkown.", self.text.r#type.position))
        }

        self.r#type.reference = Some(Rc::clone(r#use.unwrap()));

        Ok(())
    }
}
