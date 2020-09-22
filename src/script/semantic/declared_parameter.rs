
//! Module dedicated to DeclaredParameter semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::declarative_element::DeclarativeElement;
use super::r#type::Type;
use super::value::Value;

/// Structure managing and describing semantic of a declared parameter.
/// 
/// A _declared_ parameter is a parameter for which name and type are expected, as well as an optionnal value.
/// It is used by [Sequences](../sequence/struct.Sequence.html) and [Models](../model/struct.Model.html).
/// 
/// It owns the whole [text parameter](../../text/parameter/struct.Parameter.html).
pub struct DeclaredParameter {
    pub text: TextParameter,

    pub parent: Rc<RefCell<dyn DeclarativeElement>>,

    pub name: String,
    pub r#type: Type,
    pub value: Option<Rc<RefCell<Value>>>,
}

impl DeclaredParameter {
    /// Create a new semantic declared parameter, based on textual parameter.
    /// 
    /// * `parent`: the parent element that "owns" this declared parameter.
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
    /// # use melodium_rust::script::semantic::declarative_element::DeclarativeElement;
    /// # use melodium_rust::script::semantic::r#type::{TypeName, TypeStructure};
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Sequence::new(Rc::clone(&script), text_sequence),
    /// // which will itself call DeclaredParameter::new(Rc::clone(&sequence), text_parameter).
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("PrepareAudioFiles").unwrap().borrow();
    /// let borrowed_declared_parameter = borrowed_sequence.find_declared_parameter("sampleRate").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_declared_parameter.name, "sampleRate");
    /// assert_eq!(borrowed_declared_parameter.r#type.structure, TypeStructure::Scalar);
    /// assert_eq!(borrowed_declared_parameter.r#type.name, TypeName::Integer);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(parent: Rc<RefCell<dyn DeclarativeElement>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let r#type;
        let value;
        {
            let borrowed_parent = parent.borrow();

            let parameter = borrowed_parent.find_declared_parameter(&text.name.string);
            if parameter.is_some() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }

            if text.r#type.is_none() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name.string + "' do not have type.", text.name.position))
            }
            r#type = Type::new(text.r#type.as_ref().unwrap().clone())?;

            if text.value.is_some() {
                value = Some(Value::new(Rc::clone(&parent), text.value.as_ref().unwrap().clone())?);
            }
            else {
                value = None;
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            parent,
            name: text.name.string.clone(),
            text,
            r#type,
            value,
        })))
    }
}

impl Node for DeclaredParameter {
    
}

