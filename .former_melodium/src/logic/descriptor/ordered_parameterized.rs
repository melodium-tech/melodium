
use std::fmt::Debug;
use std::sync::Arc;
use super::parameter::Parameter;

pub trait OrderedParameterized : Debug + Send + Sync {
    fn parameters(&self) -> &Vec<Parameter>;
    fn as_ordered_parameterized(&self) -> Arc<dyn OrderedParameterized>;
}
