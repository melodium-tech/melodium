use super::{Attributes, DescribedType, Identifier, Parameter};
use melodium_common::descriptor::Function as CommonFunction;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub identifier: Identifier,
    pub documentation: String,
    pub parameters: Vec<Parameter>,
    pub return_type: DescribedType,
    pub attributes: Attributes,
}

impl From<&dyn CommonFunction> for Function {
    fn from(value: &dyn CommonFunction) -> Self {
        Self {
            identifier: Identifier::from(value.identifier()),
            documentation: value.documentation().to_string(),
            parameters: value
                .parameters()
                .iter()
                .map(|param| Parameter::from(param))
                .collect(),
            return_type: value.return_type().into(),
            attributes: value.attributes().into(),
        }
    }
}
