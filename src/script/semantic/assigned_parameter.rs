
//! Module dedicated to AssignedParameter semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::treatment::Treatment;
use super::value::Value;

/// Structure managing and describing semantic of an assigned parameter.
/// 
/// A _assigned_ parameter is a parameter for which name and value are expected, but _no_ type.
/// It is used by [Treatments](../treatment/struct.Treatment.html).
/// 
/// It owns the whole [text parameter](../../text/parameter/struct.Parameter.html).
pub struct AssignedParameter {
    pub text: TextParameter,

    pub treatment: Rc<RefCell<Treatment>>,

    pub name: String,
    pub value: Rc<RefCell<Value>>,
}

impl AssignedParameter {
    /// Create a new semantic assigned parameter, based on textual parameter.
    /// 
    /// * `treatment`: the parent treatment that "owns" this parameter.
    /// * `text`: the textual parameter.
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
    /// let script = Script::new(address, text_script)?;
    /// // Internally, Script::new call Sequence::new(Rc::clone(&script), text_sequence),
    /// // which will itself call Treatment::new(Rc::clone(&sequence), text_treatment),
    /// // which will then call AssignedParameter::new(Rc::clone(&treatment), text_parameter).
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("MakeHPCP").unwrap().borrow();
    /// let borrowed_treatment = borrowed_sequence.find_treatment("SpectralPeaks").unwrap().borrow();
    /// let borrowed_parameter = borrowed_treatment.find_parameter("magnitudeThreshold").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_parameter.name, "magnitudeThreshold");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(treatment: Rc<RefCell<Treatment>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let value;
        {
            let borrowed_treatment = treatment.borrow();

            let parameter = borrowed_treatment.find_parameter(&text.name);
            if parameter.is_some() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' is already assigned."))
            }

            if text.value.is_some() {
                value = Value::new(Rc::clone(&borrowed_treatment.sequence), text.value.as_ref().unwrap().clone())?;
            }
            else {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name + "' is missing value."))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            name: text.name.clone(),
            text,
            treatment,
            value,
        })))
    }
}

impl Node for AssignedParameter {
    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {

        let mut children: Vec<Rc<RefCell<dyn Node>>> = Vec::new();

        children.push(Rc::clone(&self.value) as Rc<RefCell<dyn Node>>);

        children
    }
}
