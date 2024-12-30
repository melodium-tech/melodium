use core::fmt::{Debug, Display, Formatter};
use melodium_common::{
    descriptor::{
        Attribuable, Attributes, Data as DataDescriptor, DataTrait, Documented, Identified,
        Identifier,
    },
    executive::Value,
};
use serde::de::Error;
use std::sync::Arc;

pub type FnBoundedMin = Box<dyn Fn() -> melodium_common::executive::Value + Sync + Send>;

pub type FnBoundedMax = Box<dyn Fn() -> melodium_common::executive::Value + Sync + Send>;

pub type FnFloatInfinity = Box<dyn Fn() -> melodium_common::executive::Value + Sync + Send>;

pub type FnFloatNegInfinity = Box<dyn Fn() -> melodium_common::executive::Value + Sync + Send>;

pub type FnFloatNan = Box<dyn Fn() -> melodium_common::executive::Value + Sync + Send>;

pub type FnDeserialize = Box<
    dyn Fn(
            &mut dyn erased_serde::Deserializer,
        ) -> Result<melodium_common::executive::Value, erased_serde::Error>
        + Sync
        + Send,
>;

pub struct Data {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    attributes: Attributes,
    implements: Vec<DataTrait>,

    // Trait no-value functions
    bounded_min: Option<FnBoundedMin>,
    bounded_max: Option<FnBoundedMax>,
    float_infinity: Option<FnFloatInfinity>,
    float_neg_infinity: Option<FnFloatNegInfinity>,
    float_nan: Option<FnFloatNan>,
    deserialize: Option<FnDeserialize>,
}

impl Data {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        attributes: Attributes,
        implements: Vec<DataTrait>,
        bounded_min: Option<FnBoundedMin>,
        bounded_max: Option<FnBoundedMax>,
        float_infinity: Option<FnFloatInfinity>,
        float_neg_infinity: Option<FnFloatNegInfinity>,
        float_nan: Option<FnFloatNan>,
        deserialize: Option<FnDeserialize>,
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new(Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            attributes,
            implements,
            bounded_min,
            bounded_max,
            float_infinity,
            float_neg_infinity,
            float_nan,
            deserialize,
        })
    }
}

impl Attribuable for Data {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl Identified for Data {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    fn make_use(&self, _identifier: &Identifier) -> bool {
        false
    }

    fn uses(&self) -> Vec<Identifier> {
        vec![]
    }
}

impl Documented for Data {
    fn documentation(&self) -> &str {
        #[cfg(feature = "doc")]
        {
            &self.documentation
        }
        #[cfg(not(feature = "doc"))]
        {
            &""
        }
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "data {}", self.identifier.to_string(),)?;

        Ok(())
    }
}

impl DataDescriptor for Data {
    fn implements(&self) -> &[DataTrait] {
        &self.implements
    }

    fn bounded_min(&self) -> Value {
        self.bounded_min
            .as_ref()
            .map(|func| func())
            .expect(&format!("Bounded not implemeted by {}", self.identifier))
    }

    fn bounded_max(&self) -> Value {
        self.bounded_max
            .as_ref()
            .map(|func| func())
            .expect(&format!("Bounded not implemeted by {}", self.identifier))
    }

    fn float_infinity(&self) -> Value {
        self.float_infinity
            .as_ref()
            .map(|func| func())
            .expect(&format!("Float not implemeted by {}", self.identifier))
    }

    fn float_neg_infinity(&self) -> Value {
        self.float_neg_infinity
            .as_ref()
            .map(|func| func())
            .expect(&format!("Float not implemeted by {}", self.identifier))
    }

    fn float_nan(&self) -> Value {
        self.float_nan
            .as_ref()
            .map(|func| func())
            .expect(&format!("Float not implemeted by {}", self.identifier))
    }

    fn deserialize(
        &self,
        deserializer: &mut dyn erased_serde::Deserializer,
    ) -> Result<Value, erased_serde::Error> {
        self.deserialize
            .as_ref()
            .map(|func| func(deserializer))
            .unwrap_or_else(|| {
                Err(erased_serde::Error::custom(format!(
                    "Deserialize not implemeted by {}",
                    self.identifier
                )))
            })
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("identifier", &self.identifier)
            .field("attributes", &self.attributes)
            .field("implements", &self.implements)
            .field(
                "deserialize",
                if self.deserialize.is_some() {
                    &"implemented"
                } else {
                    &"none"
                },
            )
            .finish()
    }
}
