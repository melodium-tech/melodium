use super::{Context, Data, Function, Identifier, Model, Treatment};
use melodium_common::descriptor::Entry as CommonEntry;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Entry {
    Context(Context),
    Data(Data),
    Function(Function),
    Model(Model),
    Treatment(Treatment),
}

impl Entry {
    pub fn identifier(&self) -> &Identifier {
        match self {
            Entry::Context(element) => &element.identifier,
            Entry::Data(element) => &element.identifier,
            Entry::Function(element) => &element.identifier,
            Entry::Model(element) => &element.identifier,
            Entry::Treatment(element) => &element.identifier,
        }
    }
}

impl From<&CommonEntry> for Entry {
    fn from(value: &CommonEntry) -> Self {
        match value {
            CommonEntry::Context(element) => Self::Context((&**element).into()),
            CommonEntry::Data(element) => Self::Data((&**element).into()),
            CommonEntry::Function(element) => Self::Function((&**element).into()),
            CommonEntry::Model(element) => Self::Model(element.into()),
            CommonEntry::Treatment(element) => Self::Treatment(element.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub enum EntryId {
    Context(Identifier),
    Data(Identifier),
    Function(Identifier),
    Model(Identifier),
    Treatment(Identifier),
}

impl EntryId {
    pub fn identifier(&self) -> &Identifier {
        match self {
            EntryId::Context(id) => id,
            EntryId::Data(id) => id,
            EntryId::Function(id) => id,
            EntryId::Model(id) => id,
            EntryId::Treatment(id) => id,
        }
    }
}

impl PartialOrd for EntryId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.identifier().partial_cmp(other.identifier())
    }
}

impl From<&CommonEntry> for EntryId {
    fn from(value: &CommonEntry) -> Self {
        match value {
            CommonEntry::Context(element) => Self::Context(element.identifier().into()),
            CommonEntry::Data(element) => Self::Data(element.identifier().into()),
            CommonEntry::Function(element) => Self::Function(element.identifier().into()),
            CommonEntry::Model(element) => Self::Model(element.identifier().into()),
            CommonEntry::Treatment(element) => Self::Treatment(element.identifier().into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    Context,
    Data,
    Function,
    Model,
    Treatment,
}

impl From<&CommonEntry> for EntryKind {
    fn from(value: &CommonEntry) -> Self {
        match value {
            CommonEntry::Context(_) => Self::Context,
            CommonEntry::Data(_) => Self::Data,
            CommonEntry::Function(_) => Self::Function,
            CommonEntry::Model(_) => Self::Model,
            CommonEntry::Treatment(_) => Self::Treatment,
        }
    }
}

impl From<&Entry> for EntryKind {
    fn from(value: &Entry) -> Self {
        match value {
            Entry::Context(_) => Self::Context,
            Entry::Data(_) => Self::Data,
            Entry::Function(_) => Self::Function,
            Entry::Model(_) => Self::Model,
            Entry::Treatment(_) => Self::Treatment,
        }
    }
}

impl From<&EntryId> for EntryKind {
    fn from(value: &EntryId) -> Self {
        match value {
            EntryId::Context(_) => Self::Context,
            EntryId::Data(_) => Self::Data,
            EntryId::Function(_) => Self::Function,
            EntryId::Model(_) => Self::Model,
            EntryId::Treatment(_) => Self::Treatment,
        }
    }
}
