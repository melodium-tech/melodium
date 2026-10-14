use melodium_common::executive::{
    SecretPolicy as CommonSecretPolicy, SecretReveal as CommonSecretReveal,
    SecretTransmission as CommonSecretTransmission,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SecretTransmission {
    Local,
    Reference,
    Value,
}

impl From<&CommonSecretTransmission> for SecretTransmission {
    fn from(value: &CommonSecretTransmission) -> Self {
        match value {
            CommonSecretTransmission::Local => SecretTransmission::Local,
            CommonSecretTransmission::Reference => SecretTransmission::Reference,
            CommonSecretTransmission::Value => SecretTransmission::Value,
        }
    }
}

impl Into<CommonSecretTransmission> for &SecretTransmission {
    fn into(self) -> CommonSecretTransmission {
        match self {
            SecretTransmission::Local => CommonSecretTransmission::Local,
            SecretTransmission::Reference => CommonSecretTransmission::Reference,
            SecretTransmission::Value => CommonSecretTransmission::Value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SecretReveal {
    Any,
    Only(Vec<String>),
}

impl From<&CommonSecretReveal> for SecretReveal {
    fn from(value: &CommonSecretReveal) -> Self {
        match value {
            CommonSecretReveal::Any => SecretReveal::Any,
            CommonSecretReveal::Only(identifiers) => SecretReveal::Only(identifiers.clone()),
        }
    }
}

impl Into<CommonSecretReveal> for &SecretReveal {
    fn into(self) -> CommonSecretReveal {
        match self {
            SecretReveal::Any => CommonSecretReveal::Any,
            SecretReveal::Only(identifiers) => CommonSecretReveal::Only(identifiers.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub struct SecretPolicy {
    pub transmission: SecretTransmission,
    pub reveal: SecretReveal,
    pub plain_reveal: bool,
}

impl From<&CommonSecretPolicy> for SecretPolicy {
    fn from(value: &CommonSecretPolicy) -> Self {
        Self {
            transmission: (&value.transmission).into(),
            reveal: (&value.reveal).into(),
            plain_reveal: value.plain_reveal,
        }
    }
}

impl Into<CommonSecretPolicy> for &SecretPolicy {
    fn into(self) -> CommonSecretPolicy {
        CommonSecretPolicy {
            transmission: (&self.transmission).into(),
            reveal: (&self.reveal).into(),
            plain_reveal: self.plain_reveal,
        }
    }
}
