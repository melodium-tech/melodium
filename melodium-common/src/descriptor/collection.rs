use super::{Context, Function, Identified, Identifier, Model, Treatment};
use std::collections::HashMap;
use std::sync::Arc;

pub enum Entry {
    Context(Arc<Context>),
    Function(Arc<dyn Function>),
    Model(Arc<dyn Model>),
    Treatment(Arc<dyn Treatment>),
}

impl Entry {
    pub fn identifier(&self) -> Identifier {
        match self {
            Entry::Context(c) => c.identifier().clone(),
            Entry::Function(f) => f.as_identified().identifier().clone(),
            Entry::Model(m) => m.as_identified().identifier().clone(),
            Entry::Treatment(t) => t.as_identified().identifier().clone(),
        }
    }
}

pub struct Collection {
    elements: HashMap<Identifier, Entry>,
}

impl Collection {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    pub fn identifiers(&self) -> Vec<Identifier> {
        self.elements.keys().map(|i| i.clone()).collect()
    }

    pub fn insert(&mut self, entry: Entry) {
        self.elements.insert(entry.identifier().clone(), entry);
    }

    pub fn get(&self, id: &Identifier) -> Option<&Entry> {
        self.elements.get(id)
    }

    pub fn get_tree(&self) -> Tree {
        todo!()
    }
}

pub enum Tree {
    Branch {
        name: String,
        contents: HashMap<String, Tree>,
    },
    Leaf(Entry),
}
