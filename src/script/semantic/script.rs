
//! Module dedicated to Script semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
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
pub struct Script {
    pub text: TextScript,

    pub address: String,

    pub uses: Vec<Rc<RefCell<Use>>>,
    pub models: Vec<Rc<RefCell<Model>>>,
    pub sequences: Vec<Rc<RefCell<Sequence>>>,
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
    /// let script = Script::new(address, text_script)?;
    /// 
    /// assert_eq!(script.borrow().uses.len(), 6);
    /// assert_eq!(script.borrow().models.len(), 0);
    /// assert_eq!(script.borrow().sequences.len(), 4);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(address: & str, text: TextScript) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let script = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            text: text.clone(),
            address: address.to_string(),
            uses: Vec::new(),
            models: Vec::new(),
            sequences: Vec::new(),
        }));

        for u in text.uses {
            let r#use = Use::new(Rc::clone(&script), u.clone())?;
            script.borrow_mut().uses.push(r#use);
        }

        for m in text.models {
            let model = Model::new(Rc::clone(&script), m.clone())?;
            script.borrow_mut().models.push(model);
        }

        for s in text.sequences {
            let sequence = Sequence::new(Rc::clone(&script), s.clone())?;
            script.borrow_mut().sequences.push(sequence);
        }

        Ok(script)
    }

    /// Search for an element imported through a use.
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
    /// let script = Script::new(address, text_script)?;
    /// let borrowed_script = script.borrow();
    /// 
    /// let spectrum = borrowed_script.find_use("Spectrum");
    /// let dont_exist = borrowed_script.find_use("DontExist");
    /// assert!(spectrum.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_use(&self, element: & str) -> Option<&Rc<RefCell<Use>>> {
        self.uses.iter().find(|&u| u.borrow().element == element)
    }

    /// Search for a model.
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
    /// let script = Script::new(address, text_script)?;
    /// let borrowed_script = script.borrow();
    /// 
    /// // [Sic] There is no models used in this example.
    /// let dont_exist = borrowed_script.find_model("DontExist");
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_model(&self, name: & str) -> Option<&Rc<RefCell<Model>>> {
        self.models.iter().find(|&m| m.borrow().name == name)
    }

    /// Search for a sequence.
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
    /// let script = Script::new(address, text_script)?;
    /// let borrowed_script = script.borrow();
    /// 
    /// let make_hpcp = borrowed_script.find_sequence("MakeHPCP");
    /// let dont_exist = borrowed_script.find_sequence("DontExist");
    /// assert!(make_hpcp.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_sequence(&self, name: & str) -> Option<&Rc<RefCell<Sequence>>> {
        self.sequences.iter().find(|&s| s.borrow().name == name)
    }
}

impl Node for Script {
    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {

        let mut children: Vec<Rc<RefCell<dyn Node>>> = Vec::new();

        self.uses.iter().for_each(|u| children.push(Rc::clone(&u) as Rc<RefCell<dyn Node>>));
        self.models.iter().for_each(|m| children.push(Rc::clone(&m) as Rc<RefCell<dyn Node>>));
        self.sequences.iter().for_each(|s| children.push(Rc::clone(&s) as Rc<RefCell<dyn Node>>));

        children
    }
}

#[cfg(test)]
mod tests {

    use crate::script::semantic::common::Tree;
    use crate::script_file::ScriptFile;

    #[test]
    fn test_simple_semantic() {

        let address = "examples/semantic/simple_build.mel";

        let mut script_file = ScriptFile::new(address);

        script_file.load().unwrap();
        script_file.parse().unwrap();

        let semantic_tree = Tree::new(address, script_file.script().clone());
        semantic_tree.make_references().unwrap();

        assert_eq!(semantic_tree.script.borrow().sequences.len(), 4);
    }
}