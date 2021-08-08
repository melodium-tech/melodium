
//! Module dedicated to AssignedModel semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;
use crate::script::text::Value as TextValue;

use super::assignative_element::AssignativeElement;
use super::declarative_element::DeclarativeElementType;
use super::common::Reference;
use super::declared_model::DeclaredModel;

/// Structure managing and describing semantic of an assigned model.
/// 
/// It owns the whole [text parameter](../../text/parameter/struct.Parameter.html).
pub struct AssignedModel {
    pub text: TextParameter,

    pub parent: Weak<RwLock<dyn AssignativeElement>>,

    pub name: String,
    pub model: Reference<DeclaredModel>,
}

impl AssignedModel {
    /// Create a new semantic assignation of model, based on textual parameter.
    /// 
    /// * `parent`: the parent element owning this assignation.
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
    /// # use melodium_rust::script::semantic::assignative_element::AssignativeElement;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Sequence::new(Arc::clone(&script), text_sequence),
    /// // which will itself call Treatment::new(Arc::clone(&sequence), text_treatment),
    /// // which will then call AssignedModel::new(Arc::clone(&treatment), text_parameter).
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("ReadAudioFiles").unwrap().read().unwrap();
    /// let borrowed_treatment = borrowed_sequence.find_treatment("Decoder").unwrap().read().unwrap();
    /// let borrowed_assigned_model = borrowed_treatment.find_assigned_model("AudioManager").unwrap().read().unwrap();
    /// 
    /// assert_eq!(borrowed_assigned_model.name, "AudioManager");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(parent: Arc<RwLock<dyn AssignativeElement>>, text: TextParameter) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let referred_model_name;
        {
            let borrowed_parent = parent.read().unwrap();

            let assigned_model = borrowed_parent.find_assigned_model(&text.name.string);
            if assigned_model.is_some() {
                return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' is already assigned.", text.name.position))
            }

            if let Some(erroneous_type) = &text.r#type {
                return Err(ScriptError::semantic("Model assignation cannot be typed.".to_string(), erroneous_type.name.position))
            }

            if let Some(TextValue::Name(model_name)) = &text.value {
                referred_model_name = model_name.string.clone();
            }
            else {
                return Err(ScriptError::semantic("Model assignation require a name.".to_string(), text.name.position))
            }
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            name: text.name.string.clone(),
            text,
            parent: Arc::downgrade(&parent),
            model: Reference {
                name: referred_model_name,
                reference: None,
            },
        })))
    }
}

impl Node for AssignedModel {
    fn make_references(&mut self) -> Result<(), ScriptError> {

        if self.model.reference.is_none() {
            
            let rc_parent = self.parent.upgrade().unwrap();
            let borrowed_parent = rc_parent.read().unwrap();

            let rc_declarative_element = borrowed_parent.associated_declarative_element();
            let borrowed_declarative_element = rc_declarative_element.read().unwrap();
            let refered_model = match &borrowed_declarative_element.declarative_element() {
                DeclarativeElementType::Sequence(s) => s.find_declared_model(&self.model.name),
                _ => None,
            };

            if let Some(rc_refered_model) = refered_model {
                self.model.reference = Some(Arc::downgrade(rc_refered_model));
            }
            else {
                return Err(ScriptError::semantic("Unkown name '".to_string() + &self.name + "' in declared models.", self.text.name.position));
            }
        }

        Ok(())
    }
}
