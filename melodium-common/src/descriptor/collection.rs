use super::{Context, Function, Identifier, Model, Treatment};
use std::cmp::Ordering;
use std::collections::{hash_map, HashMap};
use std::slice::Iter;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Entry {
    Context(Arc<dyn Context>),
    Function(Arc<dyn Function>),
    Model(Arc<dyn Model>),
    Treatment(Arc<dyn Treatment>),
}

impl Entry {
    pub fn identifier(&self) -> Identifier {
        match self {
            Entry::Context(c) => c.identifier().clone(),
            Entry::Function(f) => f.identifier().clone(),
            Entry::Model(m) => m.identifier().clone(),
            Entry::Treatment(t) => t.identifier().clone(),
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Context(l0), Self::Context(r0)) => l0.identifier() == r0.identifier(),
            (Self::Function(l0), Self::Function(r0)) => l0.identifier() == r0.identifier(),
            (Self::Model(l0), Self::Model(r0)) => l0.identifier() == r0.identifier(),
            (Self::Treatment(l0), Self::Treatment(r0)) => l0.identifier() == r0.identifier(),
            _ => false,
        }
    }
}

impl Eq for Entry {}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Context(l0), Self::Context(r0)) => l0.identifier().partial_cmp(r0.identifier()),
            (Self::Function(l0), Self::Function(r0)) => {
                l0.identifier().partial_cmp(r0.identifier())
            }
            (Self::Model(l0), Self::Model(r0)) => l0.identifier().partial_cmp(r0.identifier()),
            (Self::Treatment(l0), Self::Treatment(r0)) => {
                l0.identifier().partial_cmp(r0.identifier())
            }
            (Self::Context(_), _) => Some(Ordering::Less),
            (Self::Function(_), Self::Context(_)) => Some(Ordering::Greater),
            (Self::Function(_), _) => Some(Ordering::Less),
            (Self::Model(_), Self::Context(_) | Self::Function(_)) => Some(Ordering::Greater),
            (Self::Model(_), Self::Treatment(_)) => Some(Ordering::Less),
            (Self::Treatment(_), _) => Some(Ordering::Greater),
        }
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Debug)]
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

    pub fn get_tree(&self) -> CollectionTree {
        let mut tree = CollectionTree::new();

        fn insert_entry(
            tree: &mut CollectionTree,
            mut iter: Iter<String>,
            entry: Entry,
            name: String,
        ) {
            if let Some(next) = iter.next() {
                match tree.areas.entry(next.to_string()) {
                    hash_map::Entry::Occupied(mut e) => {
                        insert_entry(e.get_mut(), iter, entry, name);
                    }
                    hash_map::Entry::Vacant(v) => {
                        let mut ct = CollectionTree::new();
                        insert_entry(&mut ct, iter, entry, name);
                        v.insert(ct);
                    }
                }
            } else {
                tree.entries.push(entry);
            }
        }

        for (id, entry) in &self.elements {
            insert_entry(
                &mut tree,
                id.path().iter(),
                entry.clone(),
                id.name().to_string(),
            )
        }

        tree
    }
}

#[derive(Clone, Debug)]
pub struct CollectionTree {
    pub areas: HashMap<String, CollectionTree>,
    pub entries: Vec<Entry>,
}

impl CollectionTree {
    pub fn new() -> Self {
        Self {
            areas: HashMap::new(),
            entries: Vec::new(),
        }
    }
}
