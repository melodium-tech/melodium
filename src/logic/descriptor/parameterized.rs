
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;
use super::parameter::Parameter;

pub trait Parameterized : Debug {
    fn parameters(&self) -> &HashMap<String, Parameter>;
    fn as_parameterized(&self) -> Arc<dyn Parameterized>;
}
