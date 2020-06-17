
use crate::script::error::ScriptError;
use crate::script::text::Use as TextUse;

use super::script::Script;

pub struct Use<'a> {
    pub text: TextUse,

    pub script: &'a Script<'a>,

    pub path: Vec<String>,
    pub filePath: String,
    pub element: String,
}

impl<'a> Use<'a> {
    pub fn new(script: &'a Script, text: TextUse) -> Result<Self, ScriptError> {

        let filePath = text.path.join("/");

        let r#use = script.find_use(&text.element);
        if r#use.is_some() {
            return Err(ScriptError::semantic("'".to_string() + &text.element + "' is already used."))
        }

        Ok(Self {
            text,
            script,
            path: text.path,
            filePath,
            element: text.element,
        })
    }
}
