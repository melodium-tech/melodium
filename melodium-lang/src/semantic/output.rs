//! Module dedicated to Output semantic analysis.

use super::common::Node;
use super::r#type::Type;
use super::treatment::Treatment;
use crate::error::ScriptError;
use crate::text::Parameter as TextParameter;
use melodium_common::descriptor::Output as OutputDescriptor;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of an output.
///
/// It owns the whole [text parameter](../../text/parameter/struct.Parameter.html).
#[derive(Debug)]
pub struct Output {
    pub text: TextParameter,

    pub treatment: Weak<RwLock<Treatment>>,

    pub name: String,
    pub r#type: Type,
}

impl Output {
    /// Create a new semantic output, based on textual parameter.
    ///
    /// * `treatment`: the parent treatment that owns this output.
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
    /// # use melodium::script::semantic::r#type::{TypeName, TypeFlow, TypeStructure};
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Treatment::new(Arc::clone(&script), text_treatment),
    /// // which will itself call Output::new(Arc::clone(&treatment), text_parameter).
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("HPCP").unwrap().read().unwrap();
    /// let borrowed_output = borrowed_treatment.find_output("hpcp").unwrap().read().unwrap();
    ///
    /// assert_eq!(borrowed_output.name, "hpcp");
    /// assert_eq!(borrowed_output.r#type.flow, TypeFlow::Stream);
    /// assert_eq!(borrowed_output.r#type.structure, TypeStructure::Vector);
    /// assert_eq!(borrowed_output.r#type.name, TypeName::U64);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(
        treatment: Arc<RwLock<Treatment>>,
        text: TextParameter,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let r#type;
        {
            let borrowed_treatment = treatment.read().unwrap();

            let input = borrowed_treatment.find_output(&text.name.string);
            if input.is_some() {
                return Err(ScriptError::semantic(
                    "Output '".to_string() + &text.name.string + "' is already declared.",
                    text.name.position,
                ));
            }

            if text.r#type.is_none() {
                return Err(ScriptError::semantic(
                    "Output '".to_string() + &text.name.string + "' do not have type.",
                    text.name.position,
                ));
            }
            r#type = Type::new(text.r#type.as_ref().unwrap().clone())?;

            if text.value.is_some() {
                return Err(ScriptError::semantic(
                    "Output '".to_string() + &text.name.string + "' cannot have default value.",
                    text.name.position,
                ));
            }
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            treatment: Arc::downgrade(&treatment),
            name: text.name.string.clone(),
            text,
            r#type,
        })))
    }

    pub fn make_descriptor(&self) -> Result<OutputDescriptor, ScriptError> {
        let (datatype, flow) = self.r#type.make_descriptor()?;

        let output = OutputDescriptor::new(&self.name, datatype, flow);

        Ok(output)
    }
}

impl Node for Output {}
