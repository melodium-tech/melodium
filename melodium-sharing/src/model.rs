use super::{Attributes, Identifier, ModelDesign, Parameter, SharingError, SharingResult};
use melodium_common::descriptor::{
    Collection, Entry as CommonEntry, Identifier as CommonIdentifier, Model as CommonModel,
};
use melodium_engine::{descriptor::Model as DesignedModel, LogicError};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub identifier: Identifier,
    pub documentation: String,
    pub parameters: BTreeMap<String, Parameter>,
    pub implementation_kind: ModelImplementationKind,
    pub hierarchy: Vec<Identifier>,
    pub sources: BTreeMap<String, Vec<Identifier>>,
    pub attributes: Attributes,
}

impl Model {
    pub fn make_descriptor(&self, collection: &Collection) -> SharingResult<Arc<DesignedModel>> {
        let identifier = if let Ok(identifier) = (&self.identifier).try_into() {
            identifier
        } else {
            return SharingResult::new_failure(SharingError::invalid_identifier(
                1,
                self.identifier.clone(),
            ));
        };

        if self.implementation_kind.is_compiled() {
            return SharingResult::new_failure(SharingError::compiled_model(2, identifier));
        }

        let base_identifier: CommonIdentifier = if let Some(base_identifier) = self.hierarchy.last()
        {
            if let Ok(base_identifier) = base_identifier.try_into() {
                base_identifier
            } else {
                return SharingResult::new_failure(SharingError::invalid_identifier(
                    4,
                    self.identifier.clone(),
                ));
            }
        } else {
            return SharingResult::new_failure(SharingError::missing_base_identifier(
                3, identifier,
            ));
        };

        let base_model = if let Some(melodium_common::descriptor::Entry::Model(base_model)) =
            collection.get(&(&base_identifier).into())
        {
            base_model
        } else {
            return SharingResult::new_failure(
                LogicError::unexisting_model(230, identifier.clone(), base_identifier.into(), None)
                    .into(),
            );
        };

        let mut result = SharingResult::new_success(());
        let mut descriptor = DesignedModel::new(identifier.clone(), base_model);

        descriptor.set_documentation(&self.documentation);

        for (name, attribute) in &self.attributes.0 {
            descriptor.add_attribute(name.clone(), attribute.clone());
        }

        for (_, param) in &self.parameters {
            if let Some(parameter) =
                result.merge_degrade_failure(param.to_parameter(collection, &identifier))
            {
                descriptor.add_parameter(parameter)
            }
        }

        result.and(SharingResult::new_success(descriptor.commit()))
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> SharingResult<()> {
        let identifier = if let Ok(identifier) = (&self.identifier).try_into() {
            identifier
        } else {
            return SharingResult::new_failure(SharingError::invalid_identifier(
                6,
                self.identifier.clone(),
            ));
        };

        let design =
            if let ModelImplementationKind::Designed(Some(design)) = &self.implementation_kind {
                design
            } else {
                return SharingResult::new_failure(SharingError::no_model_design_available(
                    7, identifier,
                ));
            };

        let descriptor = if let Some(CommonEntry::Model(model)) =
            collection.get(&(&identifier).into())
        {
            if let Ok(model) = model.clone().downcast_arc::<DesignedModel>() {
                model
            } else {
                return SharingResult::new_failure(
                    LogicError::unexisting_model(235, identifier.clone(), identifier.into(), None)
                        .into(),
                );
            }
        } else {
            return SharingResult::new_failure(
                LogicError::unexisting_model(234, identifier.clone(), identifier.into(), None)
                    .into(),
            );
        };

        let designer = match descriptor.designer(Arc::clone(collection), None) {
            melodium_common::descriptor::Status::Success { success, errors: _ } => success,
            melodium_common::descriptor::Status::Failure { failure, errors: _ } => {
                return SharingResult::new_failure(failure.into())
            }
        };

        design
            .make_design(collection, &descriptor)
            .and_then(|design| {
                SharingResult::from(designer.write().unwrap().import_design(
                    &design,
                    &HashMap::new(),
                    None,
                ))
            })
    }
}

impl From<&Arc<dyn CommonModel>> for Model {
    fn from(value: &Arc<dyn CommonModel>) -> Self {
        let mut hierarchy = Vec::new();
        let mut base = value.base_model();
        while let Some(parent) = base {
            hierarchy.push(parent.identifier().into());
            base = parent.base_model();
        }
        Self {
            identifier: Identifier::from(value.identifier()),
            documentation: value.documentation().to_string(),
            parameters: value
                .parameters()
                .iter()
                .map(|(name, param)| (name.clone(), Parameter::from(param)))
                .collect(),
            implementation_kind: match value.build_mode() {
                melodium_common::descriptor::ModelBuildMode::Compiled(_) => {
                    ModelImplementationKind::Compiled
                }
                melodium_common::descriptor::ModelBuildMode::Designed() => {
                    ModelImplementationKind::Designed(
                        value
                            .clone()
                            .downcast_arc::<DesignedModel>()
                            .unwrap()
                            .design()
                            .success()
                            .map(|design| design.as_ref().into()),
                    )
                }
            },
            hierarchy,
            sources: value
                .sources()
                .iter()
                .map(|(name, contexts)| {
                    (
                        name.clone(),
                        contexts
                            .iter()
                            .map(|context| Identifier::from(context.identifier()))
                            .collect(),
                    )
                })
                .collect(),
            attributes: value.attributes().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModelImplementationKind {
    Compiled,
    Designed(Option<ModelDesign>),
}

impl ModelImplementationKind {
    pub fn is_compiled(&self) -> bool {
        match self {
            ModelImplementationKind::Compiled => true,
            _ => false,
        }
    }
}
