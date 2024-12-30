use crate::executive::Value;
use erased_serde::Deserializer;

use super::{DataTrait, Documented, Identified};
use core::fmt::{Debug, Display};

pub trait Data: Identified + Documented + Display + Debug + Send + Sync {
    fn implements(&self) -> &[DataTrait];

    // Add here specific functions for no-value traits (such as `bounded` or `deserialize`) (see #81).
    fn deserialize(
        &self,
        deserializer: &mut dyn Deserializer,
    ) -> Result<Value, erased_serde::Error>;
}

impl PartialEq for dyn Data {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}
