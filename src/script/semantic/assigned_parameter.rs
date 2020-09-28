
//! Module dedicated to AssignedParameter semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::parameter::Parameter;
use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::declarative_element::DeclarativeElement;
use super::declared_parameter::DeclaredParameter;
use super::value::Value;
use super::requirement::Requirement;

/// Structure managing and describing semantic of an assigned parameter.
/// 
/// A _assigned_ parameter is a parameter for which name and value are expected, but _no_ type.
/// It is used by [Treatments](../treatment/struct.Treatment.html) and [Models](../model/struct.Model.html).
/// 
/// It owns the whole [text parameter](../../text/parameter/struct.Parameter.html).
pub struct AssignedParameter {
    pub text: TextParameter,

    pub parent: Rc<RefCell<dyn AssignativeElement>>,

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
    /// # use melodium_rust::script::semantic::assignative_element::AssignativeElement;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Sequence::new(Rc::clone(&script), text_sequence),
    /// // which will itself call Treatment::new(Rc::clone(&sequence), text_treatment),
    /// // which will then call AssignedParameter::new(Rc::clone(&treatment), text_parameter).
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("MakeHPCP").unwrap().borrow();
    /// let borrowed_treatment = borrowed_sequence.find_treatment("SpectralPeaks").unwrap().borrow();
    /// let borrowed_parameter = borrowed_treatment.find_assigned_parameter("magnitudeThreshold").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_parameter.name, "magnitudeThreshold");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(parent: Rc<RefCell<dyn AssignativeElement>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let value;
        {
            let borrowed_parent = parent.borrow();

            let parameter = borrowed_parent.find_assigned_parameter(&text.name.string);
            if parameter.is_some() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name.string + "' is already assigned.", text.name.position))
            }

            if text.value.is_some() {

                value = Value::new(text.value.as_ref().unwrap().clone())?;
            }
            else {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name.string + "' is missing value.", text.name.position))
            }
        }

        let parameter = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            name: text.name.string.clone(),
            text,
            parent,
            value,
        }));

        parameter.borrow().value.borrow_mut().parent = Some(Rc::clone(&parameter) as Rc<RefCell<dyn Parameter>>);

        Ok(parameter)
    }
}

impl Parameter for AssignedParameter {

    fn find_declared_parameter(&self, name: & str) -> Option<Rc<RefCell<DeclaredParameter>>> {

        let borrowed_parent = self.parent.borrow();
        match borrowed_parent.assignative_element() {
            AssignativeElementType::Model(m) => {
                if let Some(param) = m.find_declared_parameter(name) {
                    Some(Rc::clone(param))
                }
                else {
                    None
                }
            }
            AssignativeElementType::Treatment(t) => {
                let sequence = t.sequence.borrow();
                if let Some(param) = sequence.find_declared_parameter(name) {
                    Some(Rc::clone(param))
                }
                else {
                    None
                }
            }
        }
    }

    fn find_requirement(&self, name: & str) -> Option<Rc<RefCell<Requirement>>> {
        
        let borrowed_parent = self.parent.borrow();
        match borrowed_parent.assignative_element() {
            AssignativeElementType::Treatment(t) => {
                let sequence = t.sequence.borrow();
                if let Some(param) = sequence.find_requirement(name) {
                    Some(Rc::clone(param))
                }
                else {
                    None
                }
            },
            _ => None
        }
    }
}

impl Node for AssignedParameter {
    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {

        let mut children: Vec<Rc<RefCell<dyn Node>>> = Vec::new();

        children.push(Rc::clone(&self.value) as Rc<RefCell<dyn Node>>);

        children
    }
}
