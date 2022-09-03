
use std::collections::{HashMap, hash_map::Entry};
use std::sync::{Arc, RwLock};
use crate::logic::descriptor::*;
use crate::logic::designer::SequenceDesigner;
use super::{sequence::Sequence, script::Script};

pub struct Tree {
    scripts: HashMap<String, Script>
}

impl Tree {

    pub fn new() -> Self {
        Self {
            scripts: HashMap::new()
        }
    }

    pub fn add_sequence(&mut self, sequence: &Arc<RwLock<SequenceDesigner>>) {
        
        let designer = sequence.read().unwrap();
        let path = designer.descriptor().identifier().path().join("/");

        match self.scripts.entry(path) {
            Entry::Occupied(mut e) => e.get_mut().add_sequence(Sequence::new(sequence)),
            Entry::Vacant(e) => {
                let mut script = Script::new();
                script.add_sequence(Sequence::new(sequence));
                e.insert(script);
            },
        }
    }

    pub fn generate(&self) -> HashMap<String, String> {

        let mut result = HashMap::new();

        for (path, script) in &self.scripts {
            result.insert(format!("{}.mel", path), script.generate());
        }

        result
    }
}
