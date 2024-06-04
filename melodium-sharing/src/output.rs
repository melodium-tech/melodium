use super::{Attributes, DescribedType, Flow};
use melodium_common::descriptor::{Attribuable, Collection, Output as CommonOutput};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub name: String,
    pub described_type: DescribedType,
    pub flow: Flow,
    pub attributes: Attributes,
}

impl Output {
    pub fn to_output(&self, collection: &Collection) -> Option<CommonOutput> {
        Some(CommonOutput::new(
            &self.name,
            (&self.described_type).to_described_type(collection)?,
            (&self.flow).into(),
            (&self.attributes).into(),
        ))
    }
}

impl From<&CommonOutput> for Output {
    fn from(value: &CommonOutput) -> Self {
        Self {
            name: value.name().to_string(),
            described_type: value.described_type().into(),
            flow: Flow::from(value.flow()),
            attributes: value.attributes().into(),
        }
    }
}

impl TryInto<CommonOutput> for Output {
    type Error = ();
    fn try_into(self) -> Result<CommonOutput, ()> {
        (&self).try_into()
    }
}

impl TryInto<CommonOutput> for &Output {
    type Error = ();
    fn try_into(self) -> Result<CommonOutput, ()> {
        Ok(CommonOutput::new(
            &self.name,
            (&self.described_type).try_into()?,
            (&self.flow).into(),
            (&self.attributes).into(),
        ))
    }
}
