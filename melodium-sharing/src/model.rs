use super::{Attributes, Identifier, ImplementationKind, Parameter};
use melodium_common::descriptor::Model as CommonModel;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub identifier: Identifier,
    pub documentation: String,
    pub parameters: BTreeMap<String, Parameter>,
    pub implementation_kind: ImplementationKind,
    pub hierarchy: Vec<Identifier>,
    pub sources: BTreeMap<String, Vec<Identifier>>,
    pub attributes: Attributes,
}

impl From<&dyn CommonModel> for Model {
    fn from(value: &dyn CommonModel) -> Self {
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
                    ImplementationKind::Compiled
                }
                melodium_common::descriptor::ModelBuildMode::Designed() => {
                    ImplementationKind::Designed
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
