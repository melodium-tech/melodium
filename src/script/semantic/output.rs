
//! Module dedicated to Output semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::sequence::Sequence;
use super::r#type::Type;

/// Structure managing and describing semantic of an output.
/// 
/// It owns the whole [text parameter](../../text/parameter/struct.Parameter.html).
pub struct Output {
    pub text: TextParameter,

    pub sequence: Rc<RefCell<Sequence>>,

    pub name: String,
    pub r#type: Type,
}

impl Output {
    /// Create a new semantic output, based on textual parameter.
    /// 
    /// * `sequence`: the parent sequence that "owns" this output.
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
    /// // which will itself call Output::new(Rc::clone(&sequence), text_parameter).
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("MakeHPCP").unwrap().borrow();
    /// let borrowed_output = borrowed_sequence.find_output("hpcp").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_output.name, "hpcp");
    /// assert_eq!(borrowed_output.r#type.structure, TypeStructure::Matrix);
    /// assert_eq!(borrowed_output.r#type.name, TypeName::Integer);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let r#type;
        {
            let borrowed_sequence = sequence.borrow();

            let input = borrowed_sequence.find_output(&text.name.string);
            if input.is_some() {
                return Err(ScriptError::semantic("Output '".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }

            if text.r#type.is_none() {
                return Err(ScriptError::semantic("Output '".to_string() + &text.name.string + "' do not have type.", text.name.position))
            }
            r#type = Type::new(text.r#type.as_ref().unwrap().clone())?;

            if text.value.is_some() {
                return Err(ScriptError::semantic("Output '".to_string() + &text.name.string + "' cannot have default value.", text.name.position))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self{
            sequence,
            name: text.name.string.clone(),
            text,
            r#type,
        })))
    }
}

impl Node for Output {
    
}
