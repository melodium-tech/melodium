
//! Module dedicated to DeclaredParameter semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;
use crate::logic::descriptor::{DataTypeDescriptor, ParameterDescriptor, FlowDescriptor};

use super::declarative_element::DeclarativeElement;
use super::r#type::Type;
use super::value::Value;

/// Structure managing and describing semantic of a declared parameter.
/// 
/// A _declared_ parameter is a parameter for which name and type are expected, as well as an optionnal value.
/// It is used by [Sequences](../sequence/struct.Sequence.html) and [Models](../model/struct.Model.html).
/// 
/// It owns the whole [text parameter](../../text/parameter/struct.Parameter.html).
#[derive(Debug)]
pub struct DeclaredParameter {
    pub text: TextParameter,

    pub parent: Weak<RwLock<dyn DeclarativeElement>>,

    pub name: String,
    pub r#type: Type,
    pub value: Option<Arc<RwLock<Value>>>,
}

impl DeclaredParameter {
    /// Create a new semantic declared parameter, based on textual parameter.
    /// 
    /// * `parent`: the parent element owning this declared parameter.
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
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("AudioToHpcpImage").unwrap().read().unwrap();
    /// let borrowed_declared_parameter = borrowed_sequence.find_declared_parameter("hopSize").unwrap().read().unwrap();
    /// 
    /// assert_eq!(borrowed_declared_parameter.name, "hopSize");
    /// assert_eq!(borrowed_declared_parameter.r#type.structure, TypeStructure::Scalar);
    /// assert_eq!(borrowed_declared_parameter.r#type.name, TypeName::Integer);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(parent: Arc<RwLock<dyn DeclarativeElement>>, text: TextParameter) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let r#type;
        let value;
        {
            let borrowed_parent = parent.read().unwrap();

            let parameter = borrowed_parent.find_declared_parameter(&text.name.string);
            if parameter.is_some() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }

            if text.r#type.is_none() {
                return Err(ScriptError::semantic("Parameter '".to_string() + &text.name.string + "' do not have type.", text.name.position))
            }
            r#type = Type::new(text.r#type.as_ref().unwrap().clone())?;

            if text.value.is_some() {
                value = Some(Value::new(Arc::clone(&parent), text.value.as_ref().unwrap().clone())?);
            }
            else {
                value = None;
            }
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            parent: Arc::downgrade(&parent),
            name: text.name.string.clone(),
            text,
            r#type,
            value,
        })))
    }

    pub fn make_descriptor(&self) -> Result<ParameterDescriptor, ScriptError> {

        let (datatype, flow) = self.r#type.make_descriptor()?;
        if flow != FlowDescriptor::Block {
            return Err(ScriptError::semantic("Parameter '".to_string() + &self.text.name.string + "' cannot have flow.", self.text.name.position));
        }

        let value = if let Some(val) = &self.value {
            Some(val.read().unwrap().make_executive_value(&datatype)?)
        }
        else {
            None
        };

        let parameter = ParameterDescriptor::new(&self.name, datatype, value);

        Ok(parameter)
    }
}

impl Node for DeclaredParameter {
    
}

