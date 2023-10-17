use melodium_common::descriptor::{Collection, Entry, Identifier, Model, Treatment};
pub use melodium_lang::ScriptResult;
use melodium_lang::{semantic::Tree as SemanticTree, text::Script as TextScript, Path};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug)]
pub enum ScriptBuildLevel {
    None,
    DescriptorsMade,
    DesignMade,
}

#[derive(Debug)]
pub struct Script {
    path: String,
    semantic: SemanticTree,
    build_level: Mutex<ScriptBuildLevel>,
}

impl Script {
    pub fn new(path: &str, text: &str) -> ScriptResult<Self> {
        match TextScript::build(&text) {
            Ok(text) => SemanticTree::new(text)
                .and_then(|tree| {
                    tree.make_references(&Path::new(
                        path.strip_suffix(".mel")
                            .unwrap_or(path)
                            .split("/")
                            .map(|s| s.to_string())
                            .collect(),
                    ))
                    .and(ScriptResult::new_success(tree))
                })
                .and_then(|tree| {
                    ScriptResult::new_success(Self {
                        path: path.to_string(),
                        semantic: tree,
                        build_level: Mutex::new(ScriptBuildLevel::None),
                    })
                }),
            Err(err) => ScriptResult::new_failure(err),
        }
    }

    pub fn build_level(&self) -> ScriptBuildLevel {
        *self.build_level.lock().unwrap()
    }

    pub fn path(&self) -> &str {
        &self.path
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

    pub fn make_descriptors(&self, collection: &mut Collection) -> ScriptResult<()> {
        let mut result = ScriptResult::new_success(());

        for (_, model) in &self.semantic.script.read().unwrap().models {
            let model = model.read().unwrap();
            if let Some(model) = result.merge_degrade_failure(model.make_descriptor(collection)) {
                collection.insert(Entry::Model(model as Arc<dyn Model>));
            }
        }

        for (_, treatment) in &self.semantic.script.read().unwrap().treatments {
            let treatment = treatment.read().unwrap();
            if let Some(treatment) =
                result.merge_degrade_failure(treatment.make_descriptor(collection))
            {
                collection.insert(Entry::Treatment(treatment as Arc<dyn Treatment>));
            }
        }

        if result.is_success() {
            *self.build_level.lock().unwrap() = ScriptBuildLevel::DescriptorsMade;
        }

        result
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> ScriptResult<()> {
        let mut result = ScriptResult::new_success(());

        for (_, model) in &self.semantic.script.read().unwrap().models {
            let model = model.read().unwrap();
            result = result.and_degrade_failure(model.make_design(collection));
        }

        for (_, treatment) in &self.semantic.script.read().unwrap().treatments {
            let treatment = treatment.read().unwrap();
            result = result.and_degrade_failure(treatment.make_design(collection));
        }

        if result.is_success() {
            *self.build_level.lock().unwrap() = ScriptBuildLevel::DesignMade;
        }

        result
    }
}
