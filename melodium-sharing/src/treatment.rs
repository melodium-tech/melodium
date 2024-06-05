use super::{Attributes, Generic, Identifier, Input, Output, Parameter, TreatmentDesign};
use melodium_common::descriptor::Treatment as CommonTreatment;
use melodium_engine::descriptor::Treatment as DesignedTreatment;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

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
