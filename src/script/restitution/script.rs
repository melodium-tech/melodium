
use std::collections::HashMap;
use convert_case::*;
use itertools::Itertools;
use super::model::Model;
use super::sequence::Sequence;
use crate::logic::descriptor::IdentifierDescriptor;

pub struct Uses {
    uses: HashMap<IdentifierDescriptor, String>
}

impl Uses {

    pub fn new(uses: HashMap<IdentifierDescriptor, String>) -> Self {
        Self {
            uses
        }
    }

    pub fn get(&self, id: &IdentifierDescriptor) -> &String {
        self.uses.get(id).unwrap()
    }
}

pub struct Script {
    path: String,
    models: Vec<Model>,
    sequences: Vec<Sequence>,
}

impl Script {

    pub fn new(path: String) -> Self {
        Self {
            path,
            models: Vec::new(),
            sequences: Vec::new(),
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn add_sequence(&mut self, sequence: Sequence) {
        self.sequences.push(sequence);
    }

    pub fn generate(&self) -> String {

        let mut uses = Vec::new();

        self.models.iter().for_each(|m| uses.extend(m.uses()));
        self.sequences.iter().for_each(|s| uses.extend(s.uses()));

        let mut uses: HashMap<IdentifierDescriptor, String> = uses.iter().unique().map(|i| (i.clone(), i.name().to_string())).collect();

        let homonymes = uses.values().duplicates().map(|s| s.clone()).collect::<Vec<_>>();
        for homonyme in homonymes {
            for (id, name) in &mut uses {
                if name == &homonyme {
                    let mut new_name = id.path().join("_").to_case(Case::UpperCamel);
                    new_name.push_str(name);

                    *name = new_name;
                }
            }
        }

        let mut result = String::new();

        for (id, name) in &uses {

            // Avoid uses of elements within the script
            if id.path().join("/") == self.path {
                continue;
            }

            result.push_str(&id.to_string());

            if id.name() != name {
                result.push_str(&format!(" as {}", name));
            }

            result.push('\n');
        }

        let uses = Uses::new(uses);

        for model in &self.models {

            result.push_str(&model.generate(&uses));
            result.push('\n');
        }

        for sequence in &self.sequences {

            result.push_str(&sequence.generate(&uses));
            result.push('\n');
        }

        result
    }
}
