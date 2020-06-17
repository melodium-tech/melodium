
use crate::script::error::ScriptError;
use crate::script::text::Script as TextScript;

use super::r#use::Use;
use super::model::Model;
use super::sequence::Sequence;

pub struct Script<'a> {
    pub text: TextScript,

    pub address: String,

    pub uses: Vec<Use<'a>>,
    pub models: Vec<Model<'a>>,
    pub sequences: Vec<Sequence<'a>>,
}

impl<'a> Script<'a> {
    pub fn new(address: & str, text: TextScript) -> Result<Self, ScriptError> {

        let mut script = Self {
            text,
            address: address.to_string(),
            uses: Vec::new(),
            models: Vec::new(),
            sequences: Vec::new(),
        };

        for u in text.uses {
            script.uses.push(Use::new(&script, u)?);
        }

        for m in text.models {
            script.models.push(Model::new(&script, m)?);
        }

        for s in text.sequences {
            script.sequences.push(Sequence::new(&script, s)?);
        }

        Ok(script)
    }

    pub fn find_use(&self, element: & str) -> Option<&Use> {
        self.uses.iter().find(|&u| u.element == element)
    }

    pub fn find_models(&self, name: & str) -> Option<&Model> {
        self.models.iter().find(|&m| m.name == name)
    }

    pub fn find_sequence(&self, name: & str) -> Option<&Sequence> {
        self.sequences.iter().find(|&s| s.name == name)
    }
}