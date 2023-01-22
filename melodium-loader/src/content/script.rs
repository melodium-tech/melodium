
use melodium_common::descriptor::{Collection, Identifier, Loader, LoadingError};
use melodium_lang::{Path, text::Script as TextScript, semantic::Tree as SemanticTree};
use std::sync::Arc;
pub use melodium_lang::ScriptError;

pub struct Script {
    path: String,
    semantic: SemanticTree,
}

impl Script {
    pub fn new(path: String, text: &str) -> Result<Self, Vec<ScriptError>> {

        let text = TextScript::build(&text).map_err(|e| vec![e])?;
        let semantic = SemanticTree::new(text).map_err(|e| vec![e])?;

        semantic.make_references(&Path::new(path.split("/").map(|s| s.to_string()).collect()));

        Ok(Self {
            path,
            semantic,
        })
    }

    pub fn match_identifier(&self, identifier: &Identifier) -> bool {
        identifier.path().join("/") == self.path
    }

    pub fn need(&self) -> Vec<Identifier> {

        let mut identifiers = Vec::new();

        for entry in &self.semantic.script.read().unwrap().uses {

            let entry = entry.read().unwrap();
            identifiers.push(entry.identifier.as_ref().unwrap().clone());
        }

        identifiers
    }

    pub fn provide(&self) -> Vec<Identifier> {

        let mut identifiers = Vec::new();

        for (_, model) in &self.semantic.script.read().unwrap().models {

            let model = model.read().unwrap();
            identifiers.push(model.identifier.as_ref().unwrap().clone());
        }

        for (_, treatment) in &self.semantic.script.read().unwrap().treatments {

            let treatment = treatment.read().unwrap();
            identifiers.push(treatment.identifier.as_ref().unwrap().clone());
        }

        identifiers
    }

    pub fn make_descriptors(&self, collection: &mut Collection) -> Result<(), Vec<ScriptError>> {

        let mut errors = Vec::new();

        for (_, model) in &self.semantic.script.read().unwrap().models {

            let model = model.read().unwrap();
            if let Err(error) = model.make_descriptor(collection) {
                errors.push(error);
            }
        }

        for (_, treatment) in &self.semantic.script.read().unwrap().treatments {

            let treatment = treatment.read().unwrap();
            if let Err(error) = treatment.make_descriptor(collection) {
                errors.push(error);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> Result<(), Vec<ScriptError>> {

        let mut errors = Vec::new();

        for (_, model) in &self.semantic.script.read().unwrap().models {

            let model = model.read().unwrap();
            if let Err(error) = model.make_design(collection) {
                errors.push(error);
            }
        }

        for (_, treatment) in &self.semantic.script.read().unwrap().treatments {

            let treatment = treatment.read().unwrap();
            if let Err(error) = treatment.make_design(collection) {
                errors.push(error);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
