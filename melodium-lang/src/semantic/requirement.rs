
//! Module dedicated to Requirement semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::error::ScriptError;
use crate::text::Requirement as TextRequirement;
use melodium_common::descriptor::requirement::Requirement as RequirementDescriptor;

use super::treatment::Treatment;

/// Structure managing and describing semantic of a requirement.
/// 
/// It owns the whole [text requirement](../../text/requirement/struct.Requirement.html).
#[derive(Debug)]
pub struct Requirement {
    pub text: TextRequirement,

    pub treatment: Weak<RwLock<Treatment>>,

    pub name: String,
}

impl Requirement {
    /// Create a new semantic requirement, based on textual requirement.
    /// 
    /// * `treatment`: the parent treatment that "owns" this requirement.
    /// * `text`: the textual requirement.
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
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Treatment::new(Arc::clone(&script), text_treatment),
    /// // which will itself call Requirement::new(Arc::clone(&treatment), text_requirement).
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("AudioToHpcpImage").unwrap().read().unwrap();
    /// let borrowed_requirement = borrowed_treatment.find_requirement("@Signal").unwrap().read().unwrap();
    /// 
    /// assert_eq!(borrowed_requirement.name, "@Signal");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(treatment: Arc<RwLock<Treatment>>, text: TextRequirement) -> Result<Arc<RwLock<Self>>, ScriptError> {

        {
            let borrowed_treatment = treatment.read().unwrap();

            let requirement = borrowed_treatment.find_requirement(&text.name.string);
            if requirement.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &text.name.string + "' is already required.", text.name.position))
            }
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self{
            treatment: Arc::downgrade(&treatment),
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
