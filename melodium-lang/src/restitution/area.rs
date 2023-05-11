use super::{model::Model, treatment::Treatment};
use crate::Path;
use convert_case::{Case, Casing};
use itertools::Itertools;
use melodium_common::descriptor::{
    Collection, Entry, Identifier, ModelBuildMode, TreatmentBuildMode,
};
use melodium_engine::descriptor::{Model as ModelDescriptor, Treatment as TreatmentDescriptor};
use std::{collections::HashMap, sync::Arc};

pub struct Area {
    path: Path,
    collection: Arc<Collection>,
    owned_ids: Vec<Identifier>,
    models: Vec<Model>,
    treatments: Vec<Treatment>,
}

impl Area {
    pub fn new(path: Path, collection: Arc<Collection>) -> Self {
        let mut owned_ids = collection
            .identifiers()
            .into_iter()
            .filter(|id| Self::is_owned(&path, &collection, id))
            .collect::<Vec<_>>();
        owned_ids.sort();

        let mut models = Vec::new();
        let mut treatments = Vec::new();

        for id in &owned_ids {
            match collection.get(id).cloned().unwrap() {
                Entry::Model(model) => {
                    models.push(Model::new(
                        model
                            .clone()
                            .downcast_arc::<ModelDescriptor>()
                            .unwrap()
                            .designer(collection.clone(), None)
                            .success()
                            .map(|designer| {
                                Arc::new(
                                    designer.read().unwrap().design().success().unwrap().clone(),
                                )
                            })
                            .unwrap_or_else(|| {
                                model
                                    .downcast_arc::<ModelDescriptor>()
                                    .unwrap()
                                    .design()
                                    .success()
                                    .unwrap()
                                    .clone()
                            }),
                    ));
                }
                Entry::Treatment(treatment) => {
                    treatments.push(Treatment::new(
                        treatment
                            .clone()
                            .downcast_arc::<TreatmentDescriptor>()
                            .unwrap()
                            .designer(collection.clone(), None)
                            .success()
                            .map(|designer| {
                                Arc::new(
                                    designer.read().unwrap().design().success().unwrap().clone(),
                                )
                            })
                            .unwrap_or_else(|| {
                                treatment
                                    .downcast_arc::<TreatmentDescriptor>()
                                    .unwrap()
                                    .design()
                                    .success()
                                    .unwrap()
                                    .clone()
                            }),
                    ));
                }
                _ => {}
            }
        }

        Self {
            path,
            collection,
            owned_ids,
            models,
            treatments,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn collection(&self) -> &Arc<Collection> {
        &self.collection
    }

    pub fn implementation(&self) -> String {
        let mut needs = Vec::new();

        self.models.iter().for_each(|m| needs.extend(m.uses()));
        self.treatments.iter().for_each(|t| needs.extend(t.uses()));
        needs.append(&mut self.owned_ids.clone());

        needs.sort();
        needs.dedup();

        let mut names: HashMap<Identifier, String> = needs
            .iter()
            .map(|id| (id.clone(), id.name().to_string()))
            .collect();
        loop {
            let conflicts = names
                .iter()
                .duplicates_by(|(_, name)| name.clone())
                .map(|(id, name)| (id.clone(), name.clone()))
                .collect::<Vec<_>>();
            if conflicts.is_empty() {
                break;
            }

            for (id, name) in conflicts {
                if !self.owned_ids.contains(&id) {
                    let new_name = Self::append_step(&id, &name);
                    names.insert(id, new_name);
                }
            }
        }

        let mut result = String::new();

        for need in &needs {
            result.push_str(&need.to_string());
            let name = names.get(need).unwrap();
            if name != need.name() {
                result.push_str(" as ");
                result.push_str(name);
            }
            result.push_str("\n");
        }

        result.push_str("\n");

        for model in &self.models {
            result.push_str(&model.implementation(&names));
        }

        for treatment in &self.treatments {
            result.push_str(&treatment.implementation(&names));
        }

        result
    }

    fn append_step(id: &Identifier, name: &str) -> String {
        let stripped_name = name.strip_prefix(&['@', '|']).unwrap_or(name);
        let prefix = name.strip_suffix(stripped_name).unwrap_or("");
        let mut full = stripped_name.to_string();
        for step in id.path().iter().rev() {
            full = format!(
                "{}{}",
                step.from_case(Case::Snake).to_case(Case::UpperCamel),
                full,
            );

            if !name.ends_with(&full) && name != full {
                break;
            }
        }

        format!("{prefix}{full}")
    }

    fn is_owned(path: &Path, collection: &Arc<Collection>, id: &Identifier) -> bool {
        if id.path() == path.path() {
            match collection.get(id).unwrap() {
                Entry::Context(_) => false,
                Entry::Function(_) => false,
                Entry::Model(model) => match model.build_mode() {
                    ModelBuildMode::Compiled(_) => false,
                    ModelBuildMode::Designed() => true,
                },
                Entry::Treatment(treatment) => match treatment.build_mode() {
                    TreatmentBuildMode::Compiled(_, _) => false,
                    TreatmentBuildMode::Source(_) => false,
                    TreatmentBuildMode::Designed() => true,
                },
            }
        } else {
            false
        }
    }
}
