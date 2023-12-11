use super::{Generic, Identified, Parameter};
use core::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;

pub trait Parameterized: Identified + Generic + Debug + Send + Sync {
    fn parameters(&self) -> &HashMap<String, Parameter>;
    fn as_identified(&self) -> Arc<dyn Identified>;
}

pub trait OrderedParameterized: Identified + Generic + Debug + Send + Sync {
    fn parameters(&self) -> &Vec<Parameter>;
    fn as_identified(&self) -> Arc<dyn Identified>;
}
