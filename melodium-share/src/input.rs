use super::{Attributes, DescribedType, Flow, SharingResult};
use melodium_common::descriptor::{
    Attribuable, Collection, Identifier as CommonIdentifier, Input as CommonInput,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Input {
    pub name: String,
    pub described_type: DescribedType,
    pub flow: Flow,
    pub attributes: Attributes,
}

impl Input {
    pub fn to_input(
        &self,
        collection: &Collection,
        scope: &CommonIdentifier,
    ) -> SharingResult<CommonInput> {
        self.described_type
            .to_described_type(collection, scope)
            .and_then(|described_type| {
                SharingResult::new_success(CommonInput::new(
                    &self.name,
                    described_type,
                    (&self.flow).into(),
                    (&self.attributes).into(),
                ))
            })
    }
}

impl From<&CommonInput> for Input {
    fn from(value: &CommonInput) -> Self {
        Self {
            name: value.name().to_string(),
            described_type: value.described_type().into(),
            flow: Flow::from(value.flow()),
            attributes: value.attributes().into(),
        }
    }
}

impl TryInto<CommonInput> for Input {
    type Error = ();
    fn try_into(self) -> Result<CommonInput, ()> {
        (&self).try_into()
    }
}

impl TryInto<CommonInput> for &Input {
    type Error = ();
    fn try_into(self) -> Result<CommonInput, ()> {
        Ok(CommonInput::new(
            &self.name,
            (&self.described_type).try_into()?,
            (&self.flow).into(),
            (&self.attributes).into(),
        ))
    }
}
