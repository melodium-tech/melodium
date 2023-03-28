//! Module dedicated to Script semantic analysis.

use super::common::Node;
use super::model::Model;
use super::r#use::Use;
use super::treatment::Treatment;
use crate::error::ScriptError;
use crate::text::Script as TextScript;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Structure managing and describing semantic of a script.
///
/// Matches the concept of a script file content.
/// It owns the whole [text script](TextScript), as well as references to semantical contained [Uses](Use), [Models](Model), and [Treatments](Treatment).
/// There is a logical coherence equivalent to the one expressed in the text script, but this coherence, as in the text, may be _incomplete_ or _broken_.
#[derive(Debug)]
pub struct Script {
    pub text: TextScript,

    pub uses: Vec<Arc<RwLock<Use>>>,
    pub models: HashMap<String, Arc<RwLock<Model>>>,
    pub treatments: HashMap<String, Arc<RwLock<Treatment>>>,
}

impl Script {
    /// Create a new semantic script, based on textual script.
    ///
    /// * `address`: the literal string specifiyng the script location (i.e. the filepath).
    /// * `text`: the textual script.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(text: TextScript) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let script = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            uses: Vec::new(),
            models: HashMap::new(),
            treatments: HashMap::new(),
        }));

        for u in text.uses {
            let r#use = Use::new(Arc::clone(&script), u.clone())?;
            script.write().unwrap().uses.push(r#use);
        }

        for m in text.models {
            let model = Model::new(Arc::clone(&script), m.clone())?;
            let name = model.read().unwrap().name.clone();
            script.write().unwrap().models.insert(name, model);
        }

        for s in text.treatments {
            let treatment = Treatment::new(Arc::clone(&script), s.clone())?;
            let name = treatment.read().unwrap().name.clone();
            script.write().unwrap().treatments.insert(name, treatment);
        }

        Ok(script)
    }

    /// Search for an element imported through a use.
    /// This search using the `as` property.
    ///
    pub fn find_use(&self, element_as: &str) -> Option<&Arc<RwLock<Use>>> {
        self.uses
            .iter()
            .find(|&u| u.read().unwrap().r#as == element_as)
    }

    /// Search for a model.
    pub fn find_model(&self, name: &str) -> Option<&Arc<RwLock<Model>>> {
        self.models.get(name)
    }

    /// Search for a treatment.
    pub fn find_treatment(&self, name: &str) -> Option<&Arc<RwLock<Treatment>>> {
        self.treatments.get(name)
    }
}

impl Node for Script {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        self.uses
            .iter()
            .for_each(|u| children.push(Arc::clone(&u) as Arc<RwLock<dyn Node>>));
        self.models
            .iter()
            .for_each(|(_, m)| children.push(Arc::clone(&m) as Arc<RwLock<dyn Node>>));
        self.treatments
            .iter()
            .for_each(|(_, s)| children.push(Arc::clone(&s) as Arc<RwLock<dyn Node>>));

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

        let address = "melodium-tests/semantic/simple_build.mel";

        let mut script_file = ScriptFile::new(address);

        script_file.load().unwrap();
        script_file.parse().unwrap();

        let semantic_tree = Tree::new(script_file.script().clone()).unwrap();
        semantic_tree.make_references().unwrap();

        assert_eq!(semantic_tree.script.borrow().treatments.len(), 4);
    }*/
}
