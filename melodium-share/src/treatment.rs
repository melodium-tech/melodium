use super::{
    Attributes, Generic, Identifier, Input, Output, Parameter, SharingError, SharingResult,
    TreatmentDesign,
};
use melodium_common::descriptor::{
    Collection, Entry as CommonEntry, Identifier as CommonIdentifier, Treatment as CommonTreatment,
};
use melodium_engine::{descriptor::Treatment as DesignedTreatment, LogicError};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Treatment {
    pub identifier: Identifier,
    pub documentation: String,
    pub generics: BTreeMap<String, Generic>,
    pub parameters: BTreeMap<String, Parameter>,
    pub implementation_kind: TreatmentImplementationKind,
    pub inputs: BTreeMap<String, Input>,
    pub outputs: BTreeMap<String, Output>,
    pub models: BTreeMap<String, Identifier>,
    pub contexts: BTreeMap<String, Identifier>,
    pub attributes: Attributes,
}

impl Treatment {
    pub fn make_descriptor(
        &self,
        collection: &Collection,
    ) -> SharingResult<Arc<DesignedTreatment>> {
        let identifier = if let Ok(identifier) = (&self.identifier).try_into() {
            identifier
        } else {
            return SharingResult::new_failure(SharingError::invalid_identifier(
                11,
                self.identifier.clone(),
            ));
        };

        if self.implementation_kind.is_compiled() {
            return SharingResult::new_failure(SharingError::compiled_treatment(12, identifier));
        }

        let mut result = SharingResult::new_success(());
        let mut descriptor = DesignedTreatment::new(identifier.clone());

        descriptor.set_documentation(&self.documentation);

        for (name, attribute) in &self.attributes.0 {
            descriptor.add_attribute(name.clone(), attribute.clone());
        }

        for (_, generic) in &self.generics {
            descriptor.add_generic(generic.into());
        }

        for (name, model) in &self.models {
            let model_identifier: CommonIdentifier = if let Ok(identifier) = model.try_into() {
                identifier
            } else {
                return SharingResult::new_failure(SharingError::invalid_identifier(
                    12,
                    self.identifier.clone(),
                ));
            };

            if let Some(CommonEntry::Model(model)) = collection.get(&(&model_identifier).into()) {
                descriptor.add_model(name, model);
            } else {
                result = result.and(SharingResult::new_failure(
                    LogicError::unexisting_model(
                        236,
                        identifier.clone(),
                        model_identifier.into(),
                        None,
                    )
                    .into(),
                ));
            }
        }

        for (_, context) in &self.contexts {
            let context_identifier: CommonIdentifier = if let Ok(identifier) = context.try_into() {
                identifier
            } else {
                return SharingResult::new_failure(SharingError::invalid_identifier(
                    13,
                    self.identifier.clone(),
                ));
            };

            if let Some(CommonEntry::Context(context)) =
                collection.get(&(&context_identifier).into())
            {
                descriptor.add_context(context);
            } else {
                result = result.and(SharingResult::new_failure(
                    LogicError::unexisting_context(
                        237,
                        identifier.clone(),
                        context_identifier.into(),
                        None,
                    )
                    .into(),
                ));
            }
        }

        for (_, param) in &self.parameters {
            if let Some(parameter) =
                result.merge_degrade_failure(param.to_parameter(collection, &identifier))
            {
                descriptor.add_parameter(parameter);
            }
        }

        for (_, input) in &self.inputs {
            if let Some(input) =
                result.merge_degrade_failure(input.to_input(collection, &identifier))
            {
                descriptor.add_input(input);
            }
        }

        for (_, output) in &self.outputs {
            if let Some(output) =
                result.merge_degrade_failure(output.to_output(collection, &identifier))
            {
                descriptor.add_output(output);
            }
        }

        result.and(SharingResult::new_success(descriptor.commit()))
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> SharingResult<()> {
        let identifier = if let Ok(identifier) = (&self.identifier).try_into() {
            identifier
        } else {
            return SharingResult::new_failure(SharingError::invalid_identifier(
                14,
                self.identifier.clone(),
            ));
        };

        let design = if let TreatmentImplementationKind::Designed(Some(design)) =
            &self.implementation_kind
        {
            design
        } else {
            return SharingResult::new_failure(SharingError::no_treatment_design_available(
                15, identifier,
            ));
        };

        let descriptor = if let Some(CommonEntry::Treatment(treatment)) =
            collection.get(&(&identifier).into())
        {
            if let Ok(treatment) = treatment.clone().downcast_arc::<DesignedTreatment>() {
                treatment
            } else {
                return SharingResult::new_failure(
                    LogicError::unexisting_treatment(
                        238,
                        identifier.clone(),
                        identifier.into(),
                        None,
                    )
                    .into(),
                );
            }
        } else {
            return SharingResult::new_failure(
                LogicError::unexisting_treatment(239, identifier.clone(), identifier.into(), None)
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
            .and_then(|()| SharingResult::from(descriptor.commit_design()))
    }
}

impl From<&Arc<dyn CommonTreatment>> for Treatment {
    fn from(value: &Arc<dyn CommonTreatment>) -> Self {
        Self {
            identifier: Identifier::from(value.identifier()),
            documentation: value.documentation().to_string(),
            generics: value
                .generics()
                .iter()
                .map(|g| (g.name.clone(), g.into()))
                .collect(),
            parameters: value
                .parameters()
                .iter()
                .map(|(name, param)| (name.clone(), Parameter::from(param)))
                .collect(),
            implementation_kind: match value.build_mode() {
                melodium_common::descriptor::TreatmentBuildMode::Compiled(_, _)
                | melodium_common::descriptor::TreatmentBuildMode::Source(_) => {
                    TreatmentImplementationKind::Compiled
                }
                melodium_common::descriptor::TreatmentBuildMode::Designed() => {
                    TreatmentImplementationKind::Designed(
                        value
                            .clone()
                            .downcast_arc::<DesignedTreatment>()
                            .unwrap()
                            .design()
                            .success()
                            .map(|design| design.as_ref().into()),
                    )
                }
            },
            inputs: value
                .inputs()
                .iter()
                .map(|(name, input)| (name.clone(), Input::from(input)))
                .collect(),
            outputs: value
                .outputs()
                .iter()
                .map(|(name, output)| (name.clone(), Output::from(output)))
                .collect(),
            models: value
                .models()
                .iter()
                .map(|(name, model)| (name.clone(), Identifier::from(model.identifier())))
                .collect(),
            contexts: value
                .contexts()
                .iter()
                .map(|(name, context)| (name.clone(), Identifier::from(context.identifier())))
                .collect(),
            attributes: value.attributes().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TreatmentImplementationKind {
    Compiled,
    Designed(Option<TreatmentDesign>),
}

impl TreatmentImplementationKind {
    pub fn is_compiled(&self) -> bool {
        match self {
            TreatmentImplementationKind::Compiled => true,
            _ => false,
        }
    }
}
