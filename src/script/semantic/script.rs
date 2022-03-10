
//! Module dedicated to Script semantic analysis.

use super::common::Node;

use std::sync::{Arc, RwLock};
use crate::script::error::ScriptError;
use crate::script::text::Script as TextScript;

use super::r#use::Use;
use super::model::Model;
use super::sequence::Sequence;

/// Structure managing and describing semantic of a script.
/// 
/// Matches the concept of a script file content.
/// It owns the whole [text script](../../text/script/struct.Script.html), as well as references to semantical contained [Uses](../use/struct.Use.html), [Models](../model/struct.Model.html), and [Sequences](../sequence/struct.Sequence.html).
/// There is a logical coherence equivalent to the one expressed in the text script, but this coherence, as in the text, may be _incomplete_ or _broken_.
#[derive(Debug)]
pub struct Script {
    pub text: TextScript,

    pub uses: Vec<Arc<RwLock<Use>>>,
    pub models: Vec<Arc<RwLock<Model>>>,
    pub sequences: Vec<Arc<RwLock<Sequence>>>,
}

impl Script {
    /// Create a new semantic script, based on textual script.
    /// 
    /// * `address`: the literal string specifiyng the script location (i.e. the filepath).
    /// * `text`: the textual script.
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
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// assert_eq!(script.read().unwrap().uses.len(), 11);
    /// assert_eq!(script.read().unwrap().models.len(), 2);
    /// assert_eq!(script.read().unwrap().sequences.len(), 5);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(text: TextScript) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let script = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            uses: Vec::new(),
            models: Vec::new(),
            sequences: Vec::new(),
        }));

        for u in text.uses {
            let r#use = Use::new(Arc::clone(&script), u.clone())?;
            script.write().unwrap().uses.push(r#use);
        }

        for m in text.models {
            let model = Model::new(Arc::clone(&script), m.clone())?;
            script.write().unwrap().models.push(model);
        }

        for s in text.sequences {
            let sequence = Sequence::new(Arc::clone(&script), s.clone())?;
            script.write().unwrap().sequences.push(sequence);
        }

        Ok(script)
    }

    /// Search for an element imported through a use.
    /// This search using the `as` property.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// let borrowed_script = script.read().unwrap();
    /// 
    /// let core_spectrum = borrowed_script.find_use("CoreSpectrum");
    /// let dont_exist = borrowed_script.find_use("DontExist");
    /// assert!(core_spectrum.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_use(&self, element_as: & str) -> Option<&Arc<RwLock<Use>>> {
        self.uses.iter().find(|&u| u.read().unwrap().r#as == element_as)
    }

    /// Search for a model.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// let borrowed_script = script.read().unwrap();
    /// 
    /// let audio_engine = borrowed_script.find_model("AudioEngine");
    /// let dont_exist = borrowed_script.find_model("DontExist");
    /// assert!(audio_engine.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_model(&self, name: & str) -> Option<&Arc<RwLock<Model>>> {
        self.models.iter().find(|&m| m.read().unwrap().name == name)
    }

    /// Search for a sequence.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// let borrowed_script = script.read().unwrap();
    /// 
    /// let hpcp = borrowed_script.find_sequence("HPCP");
    /// let dont_exist = borrowed_script.find_sequence("DontExist");
    /// assert!(hpcp.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_sequence(&self, name: & str) -> Option<&Arc<RwLock<Sequence>>> {
        self.sequences.iter().find(|&s| s.read().unwrap().name == name)
    }
}

impl Node for Script {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {

        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        self.uses.iter().for_each(|u| children.push(Arc::clone(&u) as Arc<RwLock<dyn Node>>));
        self.models.iter().for_each(|m| children.push(Arc::clone(&m) as Arc<RwLock<dyn Node>>));
        self.sequences.iter().for_each(|s| children.push(Arc::clone(&s) as Arc<RwLock<dyn Node>>));

        children
    }
}

#[cfg(test)]
mod tests {
/*
    use crate::script::semantic::common::Tree;
    use crate::script_file::ScriptFile;

    #[test]
    fn test_simple_semantic() {

        let address = "examples/semantic/simple_build.mel";

        let mut script_file = ScriptFile::new(address);

        script_file.load().unwrap();
        script_file.parse().unwrap();

        let semantic_tree = Tree::new(script_file.script().clone()).unwrap();
        semantic_tree.make_references().unwrap();

        assert_eq!(semantic_tree.script.borrow().sequences.len(), 4);
    }*/
}