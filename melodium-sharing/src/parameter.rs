use super::{Attributes, DescribedType, RawValue, Variability};
use melodium_common::descriptor::{Attribuable, Parameter as CommonParameter};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub variability: Variability,
    pub described_type: DescribedType,
    pub default: Option<RawValue>,
    pub attributes: Attributes,
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
            self.default.as_ref().map(|v| v.into()),
            (&self.attributes).into(),
        ))
    }
}
