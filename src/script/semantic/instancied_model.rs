
//! Module dedicated to InstanciedModel semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::text::Instanciation as TextInstanciation;

use super::r#use::Use;
use super::model::Model;
use super::sequence::Sequence;
use super::common::Reference;
use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_parameter::AssignedParameter;
use super::declarative_element::DeclarativeElement;

/// Structure managing and describing semantic of a model instanciation.
/// 
/// It owns the whole [text instanciation](../../text/instanciation/struct.Instanciation.html).
pub struct InstanciedModel {
    pub text: TextInstanciation,

    pub sequence: Weak<RwLock<Sequence>>,

    pub name: String,
    pub r#type: RefersTo,
    pub parameters: Vec<Arc<RwLock<AssignedParameter>>>,
}

/// Enumeration managing what model instanciation refers to.
/// 
/// This is a convenience enum, as a model instanciation may refer either on a [Use](../use/struct.Use.html) or a [Model](../model/struct.Model.html).
/// The `Unknown` variant is aimed to hold a reference-to-nothing, as long as `make_references() hasn't been called.
pub enum RefersTo {
    Unkown(Reference<()>),
    Use(Reference<Use>),
    Model(Reference<Model>),
}

impl InstanciedModel {
    /// Create a new semantic model instanciation, based on textual instanciation.
    /// 
    /// * `sequence`: the parent sequence owning this instanciation.
    /// * `text`: the textual instanciation.
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
    /// // which will itself call InstanciedModel::new(Rc::clone(&sequence), text_instanciation).
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("Main").unwrap().borrow();
    /// let borrowed_instancied_model = borrowed_sequence.find_instancied_model("Files").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_instancied_model.name, "Files");
    /// assert_eq!(borrowed_instancied_model.parameters.len(), 1);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(sequence: Arc<RwLock<Sequence>>, text: TextInstanciation) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let treatment = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            sequence: Arc::downgrade(&sequence),
            name: text.name.string.clone(),
            r#type: RefersTo::Unkown(Reference::new(text.r#type.string)),
            parameters: Vec::new(),
        }));

        {
            let borrowed_sequence = sequence.read().unwrap();

            let treatment = borrowed_sequence.find_instancied_model(&text.name.string);
            if treatment.is_some() {
                return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' is already instancied.", text.name.position))
            }
        }

        for p in text.parameters {
            let assigned_parameter = AssignedParameter::new(Arc::clone(&treatment) as Arc<RwLock<dyn AssignativeElement>>, p)?;
            treatment.write().unwrap().parameters.push(assigned_parameter);
        }

        Ok(treatment)
    }
}

impl AssignativeElement for InstanciedModel {

    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::InstanciedModel(&self)
    }

    fn associated_declarative_element(&self) -> Arc<RwLock<dyn DeclarativeElement>> {
        self.sequence.upgrade().unwrap() as Arc<RwLock<dyn DeclarativeElement>>
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
    /// let borrowed_sequence = borrowed_script.find_sequence("Main").unwrap().borrow();
    /// let borrowed_instancied_model = borrowed_sequence.find_instancied_model("Files").unwrap().borrow();
    /// 
    /// let directory = borrowed_instancied_model.find_assigned_parameter("directory");
    /// let dont_exist = borrowed_instancied_model.find_assigned_parameter("dontExist");
    /// assert!(directory.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_assigned_parameter(&self, name: & str) -> Option<&Arc<RwLock<AssignedParameter>>> {
        self.parameters.iter().find(|&a| a.read().unwrap().name == name)
    }
}

impl Node for InstanciedModel {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {

        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        self.parameters.iter().for_each(|p| children.push(Arc::clone(&p) as Arc<RwLock<dyn Node>>));

        children
    }

    fn make_references(&mut self) -> Result<(), ScriptError> {

        if let RefersTo::Unkown(reference) = &self.r#type {

            let rc_sequence = self.sequence.upgrade().unwrap();
            let borrowed_sequence = rc_sequence.read().unwrap();
            let rc_script = borrowed_sequence.script.upgrade().unwrap();
            let borrowed_script = rc_script.read().unwrap();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {

                self.r#type = RefersTo::Use(Reference{
                    name: reference.name.clone(),
                    reference: Some(Arc::downgrade(r#use.unwrap()))
                });
            }
            else {
                let model = borrowed_script.find_model(&reference.name);
                if model.is_some() {

                    self.r#type = RefersTo::Model(Reference{
                        name: reference.name.clone(),
                        reference: Some(Arc::downgrade(model.unwrap()))
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
