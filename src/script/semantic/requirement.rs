
//! Module dedicated to Requirement semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::text::Requirement as TextRequirement;
use crate::logic::descriptor::requirement::Requirement as RequirementDescriptor;

use super::sequence::Sequence;

/// Structure managing and describing semantic of a requirement.
/// 
/// It owns the whole [text requirement](../../text/requirement/struct.Requirement.html).
pub struct Requirement {
    pub text: TextRequirement,

    pub sequence: Weak<RwLock<Sequence>>,

    pub name: String,
}

impl Requirement {
    /// Create a new semantic requirement, based on textual requirement.
    /// 
    /// * `sequence`: the parent sequence that "owns" this requirement.
    /// * `text`: the textual requirement.
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
    /// // Internally, Script::new call Sequence::new(Arc::clone(&script), text_sequence),
    /// // which will itself call Requirement::new(Arc::clone(&sequence), text_requirement).
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("AudioToHpcpImage").unwrap().read().unwrap();
    /// let borrowed_requirement = borrowed_sequence.find_requirement("@Signal").unwrap().read().unwrap();
    /// 
    /// assert_eq!(borrowed_requirement.name, "@Signal");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(sequence: Arc<RwLock<Sequence>>, text: TextRequirement) -> Result<Arc<RwLock<Self>>, ScriptError> {

        {
            let borrowed_sequence = sequence.read().unwrap();

            let requirement = borrowed_sequence.find_requirement(&text.name.string);
            if requirement.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &text.name.string + "' is already required.", text.name.position))
            }
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self{
            sequence: Arc::downgrade(&sequence),
            name: text.name.string.clone(),
            text,
        })))
    }

    pub fn make_descriptor(&self) -> Result<RequirementDescriptor, ScriptError> {

        let requirement = RequirementDescriptor::new(&self.name);

        Ok(requirement)
    }
}

impl Node for Requirement {
    
}
