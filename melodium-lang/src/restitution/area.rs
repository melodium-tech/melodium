use super::{model::Model, treatment::Treatment};
use crate::Path;
use convert_case::{Case, Casing};
use itertools::Itertools;
use melodium_common::descriptor::{
    Collection, Entry, Identifier, ModelBuildMode, TreatmentBuildMode,
};
use melodium_engine::descriptor::{Model as ModelDescriptor, Treatment as TreatmentDescriptor};
use std::{collections::BTreeMap, sync::Arc};

pub struct Area {
    path: Path,
    collection: Arc<Collection>,
    owned_ids: Vec<Identifier>,
    uses_names: BTreeMap<Identifier, String>,
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
            match collection.get(&id.into()).cloned().unwrap() {
                Entry::Model(model) => {
                    let design = match model.clone().downcast_arc::<ModelDescriptor>() {
                        Ok(designed) => {
                            if let Some(designer) =
                                designed.designer(collection.clone(), None).success()
                            {
                                if let Some(design) = designer
                                    .read()
                                    .unwrap()
                                    .unvalidated_design()
                                    .success()
                                    .cloned()
                                {
                                    design
                                } else {
                                    if let Some(design) = designed.design().success() {
                                        (**design).clone()
                                    } else {
                                        continue;
                                    }
                                }
                            } else {
                                continue;
                            }
                        }
                        Err(_) => continue,
                    };
                    models.push(Model::new(design));
                }
                Entry::Treatment(treatment) => {
                    let design = match treatment.clone().downcast_arc::<TreatmentDescriptor>() {
                        Ok(designed) => {
                            if let Some(designer) =
                                designed.designer(collection.clone(), None).success()
                            {
                                if let Some(design) = designer
                                    .read()
                                    .unwrap()
                                    .unvalidated_design()
                                    .success()
                                    .cloned()
                                {
                                    design
                                } else {
                                    if let Some(design) = designed.design().success() {
                                        (**design).clone()
                                    } else {
                                        continue;
                                    }
                                }
                            } else {
                                continue;
                            }
                        }
                        Err(_) => continue,
                    };
                    treatments.push(Treatment::new(design));
                }
                _ => {}
            }
        }

        let mut needs = Vec::new();

        models.iter().for_each(|m| needs.extend(m.uses().clone()));
        treatments
            .iter()
            .for_each(|t| needs.extend(t.uses().clone()));
        needs.append(&mut owned_ids.clone());

        needs.sort();
        needs.dedup();

        let mut names: BTreeMap<Identifier, String> = needs
            .iter()
            .map(|id| (id.clone(), id.name().to_string()))
            .collect();
        loop {
            let conflicts = names
                .iter()
                .map(|(_, name)| name)
                .duplicates()
                .collect::<Vec<_>>();
            if conflicts.is_empty() {
                break;
            }

            let conflicts = names
                .iter()
                .filter_map(|(id, name)| {
                    if conflicts.contains(&name) {
                        Some((id.clone(), name.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            for (id, name) in conflicts {
                if !owned_ids.contains(&id) {
                    let new_name = Self::append_step(&id, &name);
                    names.insert(id, new_name);
                }
            }
        }

        Self {
            path,
            collection,
            owned_ids,
            uses_names: names,
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

    pub fn owned_ids(&self) -> &Vec<Identifier> {
        &self.owned_ids
    }

    pub fn uses_names(&self) -> &BTreeMap<Identifier, String> {
        &self.uses_names
    }

    pub fn models(&self) -> &Vec<Model> {
        &self.models
    }

    pub fn treatments(&self) -> &Vec<Treatment> {
        &self.treatments
    }

    pub fn implementation_needs(&self, needs: Vec<Identifier>) -> String {
        let mut result = String::new();
        for (id, name) in &self.uses_names {
            if needs.contains(id) {
                result.push_str("use ");
                result.push_str(if id.root() == &self.path.root() {
                    "root"
                } else {
                    id.root()
                });
                if id.path().len() > 1 {
                    result.push_str("/");
                }
                result.push_str(
                    &id.path()
                        .iter()
                        .skip(1)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("/"),
                );
                result.push_str("::");
                result.push_str(id.name());
                if name != id.name() {
                    result.push_str(" as ");
                    result.push_str(name);
                }
                result.push_str("\n");
            }
        }

        result.push_str("\n");

        result
    }

    pub fn implementation(&self) -> String {
        let mut result = self.implementation_needs(
            self.uses_names
                .keys()
                .filter_map(|id| {
                    if self.path.path() != id.path() {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect_vec(),
        );

        for model in &self.models {
            result.push_str(&model.implementation(&self.uses_names));
        }

        for treatment in &self.treatments {
            result.push_str(&treatment.implementation(&self.uses_names));
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
            match collection.get(&id.into()) {
                Some(Entry::Context(_)) => false,
                Some(Entry::Function(_)) => false,
                Some(Entry::Model(model)) => match model.build_mode() {
                    ModelBuildMode::Compiled(_) => false,
                    ModelBuildMode::Designed() => true,
                },
                Some(Entry::Data(_)) => false,
                Some(Entry::Treatment(treatment)) => match treatment.build_mode() {
                    TreatmentBuildMode::Compiled(_, _) => false,
                    TreatmentBuildMode::Source(_) => false,
                    TreatmentBuildMode::Designed() => true,
                },
                None => false,
            }
        } else {
            false
        }
    }
}
