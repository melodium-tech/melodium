use super::{Attributes, DescribedType, RawValue, SharingResult, Variability};
use melodium_common::descriptor::{
    Attribuable, Collection, Identifier as CommonIdentifier, Parameter as CommonParameter,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub variability: Variability,
    pub described_type: DescribedType,
    pub default: Option<RawValue>,
    pub attributes: Attributes,
}

impl Parameter {
    pub fn to_parameter(
        &self,
        collection: &Collection,
        scope: &CommonIdentifier,
    ) -> SharingResult<CommonParameter> {
        self.described_type
            .to_described_type(collection, scope)
            .and_then(|described_type| {
                let default = if let Some(val) = &self.default {
                    // TODO change when #81
                    Some(val.try_into().ok())
                } else {
                    None
                }
                .flatten();

                SharingResult::new_success(CommonParameter::new(
                    &self.name,
                    (&self.variability).into(),
                    described_type,
                    default,
                    (&self.attributes).into(),
                ))
            })
    }
}

impl From<&CommonParameter> for Parameter {
    fn from(value: &CommonParameter) -> Self {
        Self {
            name: value.name().to_string(),
            variability: Variability::from(value.variability()),
            described_type: value.described_type().into(),
            default: value
                .default()
                .clone()
                .map(|v| RawValue::try_from(v).ok())
                .flatten(),
            attributes: value.attributes().into(),
        }
    }
}

impl TryInto<CommonParameter> for Parameter {
    type Error = ();
    fn try_into(self) -> Result<CommonParameter, ()> {
        (&self).try_into()
    }
}

impl TryInto<CommonParameter> for &Parameter {
    type Error = ();
    fn try_into(self) -> Result<CommonParameter, ()> {
        Ok(CommonParameter::new(
            &self.name,
            (&self.variability).into(),
            (&self.described_type).try_into()?,
            if let Some(val) = &self.default {
                Some(val.try_into()?)
            } else {
                None
            },
            (&self.attributes).into(),
        ))
    }
}
