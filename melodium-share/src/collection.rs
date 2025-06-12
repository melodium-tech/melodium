use crate::{Context, Data, Function, Identifier, Model, Treatment};
use itertools::Itertools;
use melodium_common::descriptor::{
    Collection as CommonCollection, Entry, Identified, Identifier as CommonIdentifier,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    Context(Context),
    Data(Data),
    Function(Function),
    Model(Model),
    Treatment(Treatment),
}

impl Element {
    pub fn identifier(&self) -> &Identifier {
        match self {
            Element::Context(c) => &c.identifier,
            Element::Data(d) => &d.identifier,
            Element::Function(f) => &f.identifier,
            Element::Model(m) => &m.identifier,
            Element::Treatment(t) => &t.identifier,
        }
    }

    pub fn is_compiled(&self) -> bool {
        match self {
            Element::Context(_) => true,
            Element::Data(_) => true,
            Element::Function(_) => true,
            Element::Model(m) => m.implementation_kind.is_compiled(),
            Element::Treatment(t) => t.implementation_kind.is_compiled(),
        }
    }
}

impl From<&Entry> for Element {
    fn from(value: &Entry) -> Self {
        match value {
            Entry::Context(c) => Element::Context(c.as_ref().into()),
            Entry::Data(d) => Element::Data(d.as_ref().into()),
            Entry::Function(f) => Element::Function(f.as_ref().into()),
            Entry::Model(m) => Element::Model(m.into()),
            Entry::Treatment(t) => Element::Treatment(t.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    elements: Vec<Element>,
}

impl Collection {
    pub fn elements(&self) -> &Vec<Element> {
        &self.elements
    }

    pub fn to_elements(self) -> Vec<Element> {
        self.elements
    }

    pub fn from_entrypoint(collection: &CommonCollection, entrypoint: &CommonIdentifier) -> Self {
        let mut identifiers: VecDeque<CommonIdentifier> = VecDeque::new();

        if let Some(element) = collection.get(&entrypoint.into()) {
            let mut prepending_identifiers = element.uses();
            while !prepending_identifiers.is_empty() {
                prepending_identifiers
                    .iter()
                    .rev()
                    .for_each(|id| identifiers.push_front(id.clone()));

                let mut new_identifiers = Vec::new();
                for id in &prepending_identifiers {
                    if let Some(element) = collection.get(&id.into()) {
                        new_identifiers.extend(element.uses());
                    }
                }

                prepending_identifiers = new_identifiers;
            }
        }

        identifiers.push_back(entrypoint.clone());

        let elements = identifiers
            .iter()
            .unique()
            .filter_map(|identifier| collection.get(&identifier.into()).map(|entry| entry.into()))
            .collect();

        Self { elements }
    }
}

impl From<&CommonCollection> for Collection {
    fn from(collection: &CommonCollection) -> Self {
        Self {
            elements: collection
                .identifiers()
                .iter()
                .filter_map(|identifier| {
                    collection
                        .get(&(identifier.into()))
                        .map(|entry| entry.into())
                })
                .collect(),
        }
    }
}
