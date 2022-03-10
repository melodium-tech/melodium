
//! Module dedicated to InstanciedModel semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::{ScriptError, wrap_logic_error};
use crate::script::path::Path;
use crate::script::text::Instanciation as TextInstanciation;
use crate::logic::descriptor::identifier::Identifier;
use crate::logic::designer::ModelInstanciationDesigner;

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
#[derive(Debug)]
pub struct InstanciedModel {
    pub text: TextInstanciation,

    pub sequence: Weak<RwLock<Sequence>>,

    pub name: String,
    pub r#type: RefersTo,
    pub parameters: Vec<Arc<RwLock<AssignedParameter>>>,

    pub type_identifier: Option<Identifier>,
}

/// Enumeration managing what model instanciation refers to.
/// 
/// This is a convenience enum, as a model instanciation may refer either on a [Use](../use/struct.Use.html) or a [Model](../model/struct.Model.html).
/// The `Unknown` variant is aimed to hold a reference-to-nothing, as long as `make_references() hasn't been called.
#[derive(Debug)]
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
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Sequence::new(Arc::clone(&script), text_sequence),
    /// // which will itself call InstanciedModel::new(Arc::clone(&sequence), text_instanciation).
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("Main").unwrap().read().unwrap();
    /// let borrowed_instancied_model = borrowed_sequence.find_instancied_model("Files").unwrap().read().unwrap();
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
            type_identifier: None,
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

    pub fn make_design(&self, designer: &Arc<RwLock<ModelInstanciationDesigner>>) -> Result<(), ScriptError> {

        let mut designer = designer.write().unwrap();

        for rc_assignation in &self.parameters {

            let borrowed_assignation = rc_assignation.read().unwrap();

            let assignation_designer = wrap_logic_error!(
                designer.add_parameter(&borrowed_assignation.name),
                borrowed_assignation.text.name.position
            );

            borrowed_assignation.make_design(&assignation_designer)?;
        }

        wrap_logic_error!(designer.validate(), self.text.name.position);

        Ok(())

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
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// # use melodium::script::semantic::assignative_element::AssignativeElement;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("Main").unwrap().read().unwrap();
    /// let borrowed_instancied_model = borrowed_sequence.find_instancied_model("Files").unwrap().read().unwrap();
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

    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {

        if let RefersTo::Unkown(reference) = &self.r#type {

            let rc_sequence = self.sequence.upgrade().unwrap();
            let borrowed_sequence = rc_sequence.read().unwrap();
            let rc_script = borrowed_sequence.script.upgrade().unwrap();
            let borrowed_script = rc_script.read().unwrap();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {

                let r#use = r#use.unwrap();

                self.type_identifier = r#use.read().unwrap().identifier.clone();

                self.r#type = RefersTo::Use(Reference{
                    name: reference.name.clone(),
                    reference: Some(Arc::downgrade(r#use))
                });
            }
            else {
                let model = borrowed_script.find_model(&reference.name);
                if model.is_some() {

                    self.type_identifier = path.to_identifier(&reference.name);

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
