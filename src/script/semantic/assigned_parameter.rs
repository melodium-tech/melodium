
//! Module dedicated to AssignedParameter semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::{ScriptError, wrap_logic_error};
use crate::script::text::Parameter as TextParameter;
use crate::logic::designer::ParameterDesigner;

use super::assignative_element::AssignativeElement;
use super::value::Value;

/// Structure managing and describing semantic of an assigned parameter.
/// 
/// A _assigned_ parameter is a parameter for which name and value are expected, but _no_ type.
/// It is used by [Treatments](../treatment/struct.Treatment.html) and [Models](../model/struct.Model.html).
/// 
/// It owns the whole [text parameter](../../text/parameter/struct.Parameter.html).
#[derive(Debug)]
pub struct AssignedParameter {
    pub text: TextParameter,

    pub parent: Weak<RwLock<dyn AssignativeElement>>,

    pub name: String,
    pub value: Arc<RwLock<Value>>,
}

impl AssignedParameter {
    /// Create a new semantic assigned parameter, based on textual parameter.
    /// 
    /// * `parent`: the parent owning this parameter.
    /// * `text`: the textual parameter.
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
    /// # use melodium::script::semantic::assignative_element::AssignativeElement;
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
    /// // which will then call AssignedParameter::new(Arc::clone(&treatment), text_parameter).
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("HPCP").unwrap().read().unwrap();
    /// let borrowed_treatment = borrowed_sequence.find_treatment("CoreSpectralPeaks").unwrap().read().unwrap();
    /// let borrowed_parameter = borrowed_treatment.find_assigned_parameter("magnitudeThreshold").unwrap().read().unwrap();
    /// 
    /// assert_eq!(borrowed_parameter.name, "magnitudeThreshold");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(parent: Arc<RwLock<dyn AssignativeElement>>, text: TextParameter) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let value;
        {
            let borrowed_parent = parent.read().unwrap();

            let parameter = borrowed_parent.find_assigned_parameter(&text.name.string);
            if parameter.is_some() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name.string + "' is already assigned.", text.name.position))
            }

            if text.value.is_some() {

                value = Value::new(borrowed_parent.associated_declarative_element(), text.value.as_ref().unwrap().clone())?;
            }
            else {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name.string + "' is missing value.", text.name.position))
            }
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            name: text.name.string.clone(),
            text,
            parent: Arc::downgrade(&parent),
            value,
        })))
    }

    pub fn make_design(&self, designer: &Arc<RwLock<ParameterDesigner>>) -> Result<(), ScriptError> {

        let mut designer = designer.write().unwrap();
        let descriptor = designer.parent_descriptor().upgrade().unwrap().parameters().get(&self.name).unwrap().clone();

        let value = self.value.read().unwrap().make_designed_value(descriptor.datatype())?;

        wrap_logic_error!(
            designer.set_value(value),
            self.text.name.position
        );

        Ok(())

    }
}

impl Node for AssignedParameter {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {

        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        children.push(Arc::clone(&self.value) as Arc<RwLock<dyn Node>>);

        children
    }
}
