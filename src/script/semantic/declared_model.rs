
//! Module dedicated to DeclaredModel semantic analysis.

use super::common::Node;

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;
use crate::script::text::word::PositionnedString;

use super::common::Reference;
use super::sequence::Sequence;
use super::r#use::Use;
use super::instancied_model::InstanciedModel;

/// Structure managing and describing semantic of a declared model.
/// 
/// It owns optionnally the whole [text parameter](../../text/parameter/struct.Parameter.html),
/// depending on explicit or implicit declaration.
pub struct DeclaredModel {
    pub text: Option<TextParameter>,

    pub sequence: Weak<RefCell<Sequence>>,

    pub name: String,
    pub refers: RefersTo,
}

/// Enumeration managing what declared model type refers to.
/// 
/// This is a convenience enum, as a declared model type may refer either on a [Use](../use/struct.Use.html) or an [InstanciedModel](../instancied_model/struct.InstanciedModel.html).
/// The `Unknown` variant is aimed to hold a reference-to-nothing, as long as `make_references() hasn't been called.

pub enum RefersTo {
    Unkown(Reference<()>),
    Use(Reference<Use>),
    InstanciedModel(Reference<InstanciedModel>),
}

impl DeclaredModel {
    /// Create a new semantic declaration of model, from an instancied model.
    /// 
    /// When using this creation method, the `text` member will be `None`.
    /// 
    /// * `instancied_model`: the InstanciedModel to use as declaration.
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
    /// // Internally, Script::new call Sequence::new(Rc::clone(&script), text_sequence),
    /// // which will itself call DeclaredModel::from_instancied_model(Rc::clone(&instancied_model)).
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("Main").unwrap().borrow();
    /// let borrowed_declared_model = borrowed_sequence.find_declared_model("Files").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_declared_model.name, "Files");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn from_instancied_model(instancied_model: Rc<RefCell<InstanciedModel>>) -> Result<Rc<RefCell<Self>>, ScriptError> {
        
        let borrowed_instancied_model = instancied_model.borrow();

        let sequence = borrowed_instancied_model.sequence.upgrade().unwrap();
        let name = borrowed_instancied_model.name.clone();

        let declared_model = Self::make(sequence, borrowed_instancied_model.text.name.clone())?;

        declared_model.borrow_mut().refers = RefersTo::InstanciedModel(Reference {
            name: name,
            reference: Some(Rc::downgrade(&instancied_model))
        });

        Ok(declared_model)
    }

    /// Create a new semantic declaration of model, based on textual parameter.
    /// 
    /// * `sequence`: the sequence owning this declaration.
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
    /// // Internally, Script::new call Sequence::new(Rc::clone(&script), text_sequence),
    /// // which will itself call DeclaredModel::new(Rc::clone(&sequence), text_parameter).
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("AudioToHpcpImage").unwrap().borrow();
    /// let borrowed_declared_model = borrowed_sequence.find_declared_model("AudioManager").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_declared_model.name, "AudioManager");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let refers_string;
        if let Some(r#type) = &text.r#type {

            if r#type.structure.is_some() {
                return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' cannot have type structure.", text.name.position))
            }

            refers_string = r#type.name.string.clone();
        }
        else {
            return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' do not have type.", text.name.position))
        }

        if text.value.is_some() {
            return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' cannot be assigned to a value.", text.name.position))
        }

        let declared_model = Self::make(sequence, text.name.clone())?;

        declared_model.borrow_mut().text = Some(text);
        declared_model.borrow_mut().refers = RefersTo::Unkown(Reference::new(refers_string));

        Ok(declared_model)
    }

    fn make(sequence: Rc<RefCell<Sequence>>, name: PositionnedString) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let borrowed_sequence = sequence.borrow();

        let declared_model = borrowed_sequence.find_declared_model(&name.string.clone());
        if declared_model.is_some() {
            return Err(ScriptError::semantic("Model '".to_string() + &name.string.clone() + "' is already declared.", name.position.clone()))
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            sequence: Rc::downgrade(&sequence),
            name: name.string.clone(),
            text: None,
            refers: RefersTo::Unkown(Reference::new(name.string))
        })))
    }
}

impl Node for DeclaredModel {
    fn make_references(&mut self) -> Result<(), ScriptError> {
        
        // Reference to an instancied model already been done through Self::from_instancied_model
        // so we only look for reference to a use.
        if let RefersTo::Unkown(reference) = &self.refers {

            let rc_sequence = self.sequence.upgrade().unwrap();
            let borrowed_sequence = rc_sequence.borrow();
            let rc_script = borrowed_sequence.script.upgrade().unwrap();
            let borrowed_script = rc_script.borrow();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {

                self.refers = RefersTo::Use(Reference{
                    name: reference.name.clone(),
                    reference: Some(Rc::downgrade(r#use.unwrap()))
                });
            }
            else {
                return Err(ScriptError::semantic("'".to_string() + &reference.name + "' is unkown.", self.text.as_ref().unwrap().r#type.as_ref().unwrap().name.position))
            }
        }

        Ok(())
    }
}
