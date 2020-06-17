
use crate::script::error::ScriptError;
use crate::script::text::Model as TextModel;

use super::script::Script;
use super::r#use::Use;

pub struct Model<'a> {
    pub text: TextModel,

    pub script: &'a Script<'a>,

    pub name: String,
    pub r#type: String,
    pub refers: &'a Use<'a>,
}

impl<'a> Model<'a> {
    pub fn new(script: &'a Script, text: TextModel) -> Result<Self, ScriptError> {

        let r#use = script.find_use(&text.r#type);
        if r#use.is_none() {
            return Err(ScriptError::semantic("'".to_string() + &text.r#type + "' is unkown."))
        }

        let model = script.find_models(&text.name);
        if model.is_some() {
            return Err(ScriptError::semantic("'".to_string() + &text.name+ "' is already declared."))
        }

        Ok(Self {
            text,
            script,
            name: text.name,
            r#type: text.r#type,
            refers: r#use.unwrap()
        })
    }
}
